//! The inline cache implementation for the AVM2, for speeding up property access.

use super::{object::Shape, property::Property, Error, TObject, Value};
use gc_arena::Collect;

const IC_SIZE: usize = 8;

#[derive(Debug, Clone, Collect, Default)]
#[collect(no_drop)]
pub struct InlineCache<'gc, V>
where
    V: PartialEq + Copy,
{
    entries: [(Option<Shape<'gc>>, Option<V>); IC_SIZE],
    next_slot: usize,
}

impl<'gc, V> InlineCache<'gc, V>
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
    pub fn lookup(&self, shape: &Shape<'gc>) -> Option<&V> {
        for (cache_shape, value) in &self.entries {
            if let (Some(s), Some(v)) = (cache_shape, value) {
                if s == shape {
                    return Some(v);
                }
            }
        }
        None
    }

    pub fn insert(&mut self, shape: Shape<'gc>, value: V) {
        self.entries[self.next_slot] = (Some(shape), Some(value));
        self.next_slot = (self.next_slot + 1) % IC_SIZE;
    }
}

impl<'gc> InlineCache<'gc, Property> {
    #[inline(always)]
    pub fn lookup_value_with_object<T>(
        &mut self,
        object: T,
    ) -> Result<Option<Value<'gc>>, Error<'gc>>
    where
        T: TObject<'gc>,
    {
        let base = object.base();
        let shape = base.shape();
        if let Some(shape) = *shape {
            let last_idx = (self.next_slot + IC_SIZE - 1) % IC_SIZE;
            if let (Some(s), Some(prop)) = (&self.entries[last_idx].0, &self.entries[last_idx].1) {
                if s == &shape {
                    return Self::resolve_property(object, *prop);
                }
            }

            if let Some(prop) = self.lookup(&shape) {
                return Self::resolve_property(object, *prop);
            }
        }
        Ok(None)
    }

    #[inline(always)]
    fn resolve_property<T>(object: T, property: Property) -> Result<Option<Value<'gc>>, Error<'gc>>
    where
        T: TObject<'gc>,
    {
        match property {
            Property::Method { disp_id } => object
                .get_bound_method(disp_id)
                .map_or(Ok(None), |bound_method| Ok(Some(bound_method.into()))),
            Property::Slot { slot_id } | Property::ConstSlot { slot_id } => {
                Ok(Some(object.base().get_slot(slot_id)))
            }
            _ => Ok(None),
        }
    }
}
