//! The inline cache implementation for the AVM2, for speeding up property access.

use super::{object::Shape, property::Property, Activation, Error, TObject, Value};
use gc_arena::Collect;

const IC_SIZE: usize = 4;

#[derive(Debug, Clone, Collect, Default)]
#[collect(no_drop)]
pub struct InlineCache<'gc, V>
where
    V: PartialEq,
{
    ic: Vec<(Shape<'gc>, V)>, // inline cache, keyed by shape, and value by actual property
}

impl<'gc, V> InlineCache<'gc, V>
where
    V: PartialEq,
{
    pub fn new() -> Self {
        Self { ic: Vec::new() }
    }

    pub fn lookup(&self, shape: &Shape<'gc>) -> Option<&V> {
        self.ic
            .iter()
            .find_map(|(s, v)| (*s == *shape).then_some(v))
    }

    pub fn insert(&mut self, shape: Shape<'gc>, value: V) {
        if self.ic.len() >= IC_SIZE {
            self.ic.pop();
        }
        self.ic.push((shape, value));
    }
}

impl<'gc> InlineCache<'gc, Property> {
    pub fn lookup_value_with_object<T>(
        &self,
        object: T,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<Value<'gc>>, Error<'gc>>
    where
        T: TObject<'gc>,
    {
        let base = object.base();
        let shape = base.shape();
        let property = shape.and_then(|shape| self.lookup(&shape)).copied();

        if let Some(property) = property {
            match property {
                Property::Virtual { get, .. } => get.map_or(Ok(None), |getter| {
                    object.call_method(getter, &[], activation).map(Some)
                }),
                Property::Method { disp_id } => {
                    // TODO: this is not a complete implementation, but suffice for a fast path
                    object
                        .get_bound_method(disp_id)
                        .map_or(Ok(None), |bound_method| Ok(Some(bound_method.into())))
                }
                Property::Slot { slot_id } | Property::ConstSlot { slot_id } => {
                    Ok(Some(base.get_slot(slot_id)))
                }
            }
        } else {
            Ok(None)
        }
    }
}
