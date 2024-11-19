//! The inline cache implementation for the AVM2, for speeding up property access.

use super::{object::Shape, property::Property, Error, TObject, Value};
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
        // Slow path: Perform a full lookup
        let property = shape.and_then(|s| self.lookup(&s)).copied();

        if let Some(property) = property {
            return Self::resolve_property(object, property);
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
            _ => Ok(None), // Removed due to ownership issues
        }
    }
}
