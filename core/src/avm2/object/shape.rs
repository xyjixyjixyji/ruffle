use std::marker::PhantomData;

use gc_arena::Collect;

/// Hidden class for 'ScriptObject'
///
/// It stores the property informations of an object, for example
/// an object with a field 'x' will have field 'x' in its local storage,
/// which is the slot. The shape will store the property name and the slot id.
/// For a property lookup, it inherently looks up the slot id and read/write
/// to the local storage.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Collect)]
#[collect(no_drop)]
pub struct Shape<'gc> {
    _marker: PhantomData<&'gc ()>,
}

impl<'gc> Shape<'gc> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
