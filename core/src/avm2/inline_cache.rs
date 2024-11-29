//! The inline cache implementation for the AVM2, for speeding up property access.

use super::{property::Property, Activation, Error, Multiname, TObject, Value};
use gc_arena::Collect;

const IC_SIZE: usize = 8;

#[derive(Debug, Clone, Collect, Default)]
#[collect(no_drop)]
pub struct InlineCache<V>
where
    V: PartialEq + Copy,
{
    entries: [(Option<usize>, Option<V>); IC_SIZE],
    next_slot: usize,
}

impl<V> InlineCache<V>
where
    V: PartialEq + Copy,
{
    pub fn new() -> Self {
        Self {
            entries: [(None, None); IC_SIZE],
            next_slot: 0,
        }
    }

    #[inline(always)]
    pub fn lookup(&self, shape_id: usize) -> Option<&V> {
        for (cache_shape_id, value) in &self.entries {
            if let (Some(id), Some(v)) = (cache_shape_id, value) {
                if *id == shape_id {
                    return Some(v);
                }
            }
        }
        None
    }

    #[inline(always)]
    pub fn insert(&mut self, shape_id: usize, value: V) {
        self.entries[self.next_slot] = (Some(shape_id), Some(value));
        self.next_slot = (self.next_slot + 1) % IC_SIZE;
    }
}

impl<'gc> InlineCache<Property> {
    #[inline(always)]
    pub fn lookup_value_with_object<T>(
        &mut self,
        object: T,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<Value<'gc>>, Error<'gc>>
    where
        T: TObject<'gc>,
    {
        let base = object.base();
        let shape_id = base.shape_id();
        if let Some(shape_id) = shape_id {
            let last_idx = (self.next_slot + IC_SIZE - 1) % IC_SIZE;
            if let (Some(s), Some(prop)) = (&self.entries[last_idx].0, &self.entries[last_idx].1) {
                if *s == shape_id {
                    return Self::resolve_property(object, *prop, activation);
                }
            }

            if let Some(prop) = self.lookup(shape_id) {
                return Self::resolve_property(object, *prop, activation);
            }
        }
        Ok(None)
    }

    #[inline(always)]
    pub fn call_function_with_object<T>(
        &mut self,
        object: T,
        arguments: &[Value<'gc>],
        multiname: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<Value<'gc>>, Error<'gc>>
    where
        T: TObject<'gc>,
    {
        let base = object.base();
        let shape_id = base.shape_id();
        if let Some(shape_id) = shape_id {
            if let Some(prop) = self.lookup(shape_id) {
                return Self::call_property_with_object(
                    object, *prop, arguments, multiname, activation,
                );
            }
        }
        Ok(None)
    }

    #[inline(always)]
    fn call_property_with_object<T>(
        receiver: T,
        property: Property,
        arguments: &[Value<'gc>],
        multiname: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<Value<'gc>>, Error<'gc>>
    where
        T: TObject<'gc>,
    {
        match property {
            Property::Slot { slot_id } | Property::ConstSlot { slot_id } => {
                let obj = receiver.base().get_slot(slot_id).as_callable(
                    activation,
                    Some(multiname),
                    Some(Value::from(receiver.into())),
                    false,
                )?;
                obj.call(Value::from(receiver.into()), arguments, activation)
                    .map(Some)
            }
            Property::Method { disp_id } => receiver
                .call_method(disp_id, arguments, activation)
                .map(Some),
            Property::Virtual { get: Some(get), .. } => {
                let obj = receiver.call_method(get, &[], activation)?.as_callable(
                    activation,
                    Some(multiname),
                    Some(Value::from(receiver.into())),
                    false,
                )?;
                obj.call(Value::from(receiver.into()), arguments, activation)
                    .map(Some)
            }
            _ => Ok(None),
        }
    }

    #[inline(always)]
    fn resolve_property<T>(
        object: T,
        property: Property,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<Value<'gc>>, Error<'gc>>
    where
        T: TObject<'gc>,
    {
        match property {
            Property::Slot { slot_id } | Property::ConstSlot { slot_id } => {
                Ok(Some(object.base().get_slot(slot_id)))
            }
            Property::Virtual { get: Some(get), .. } => {
                object.call_method(get, &[], activation).map(Some)
            }
            Property::Method { disp_id } => object
                .get_bound_method(disp_id)
                .map_or(Ok(None), |bound_method| Ok(Some(bound_method.into()))),
            _ => Ok(None),
        }
    }
}
