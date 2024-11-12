use gc_arena::{Collect, GcCell, Mutation};

use crate::{
    avm2::{property::Property, vtable::VTable, Namespace},
    string::AvmString,
};

#[derive(Debug, Clone, Copy, PartialEq, Collect)]
#[collect(no_drop)]
pub struct PropertyInfo<'gc> {
    name: AvmString<'gc>,
    ns: Namespace<'gc>,
    property: Property,
}

/// Hidden class for 'ScriptObject'
///
/// It stores the property informations of an object, for example
/// an object with a field 'x' will have field 'x' in its local storage,
/// which is the slot. The shape will store the property name and the slot id.
/// For a property lookup, it inherently looks up the slot id and read/write
/// to the local storage.
#[derive(Debug, Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct Shape<'gc>(GcCell<'gc, ShapeData<'gc>>);

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct ShapeData<'gc> {
    /// Properties in this shape, keyed by local name, value is the slot id.
    pub(crate) properties: Vec<PropertyInfo<'gc>>,
}

impl<'gc> Shape<'gc> {
    pub fn empty(mc: &Mutation<'gc>) -> Self {
        Self(GcCell::new(mc, ShapeData { properties: vec![] }))
    }

    pub fn new(mc: &Mutation<'gc>, vtable: &VTable<'gc>) -> Self {
        Self(GcCell::new(
            mc,
            ShapeData {
                properties: vtable
                    .resolved_traits()
                    .iter()
                    .map(|(name, ns, property)| PropertyInfo {
                        name: name.clone(),
                        ns: ns.clone(),
                        property: *property,
                    })
                    .collect(),
            },
        ))
    }
}
