use std::borrow::Borrow;

use gc_arena::{Collect, GcCell, Mutation};

use crate::{
    avm2::{property::Property, vtable::VTable, Multiname, Namespace, Value},
    string::AvmString,
};

#[derive(Debug, Clone, PartialEq, Collect)]
#[collect(no_drop)]
pub enum PropertyType<'gc> {
    Property(Property),
    Value(Value<'gc>),
}

#[derive(Debug, Clone, PartialEq, Collect)]
#[collect(no_drop)]
pub struct PropertyInfo<'gc> {
    name: AvmString<'gc>,
    ns: Vec<Namespace<'gc>>,
    property: PropertyType<'gc>,
}

impl<'gc> PropertyInfo<'gc> {
    pub fn new(name: AvmString<'gc>, ns: Vec<Namespace<'gc>>, property: PropertyType<'gc>) -> Self {
        Self { name, ns, property }
    }

    pub fn name(&self) -> AvmString<'gc> {
        self.name
    }

    pub fn ns(&self) -> &[Namespace<'gc>] {
        &self.ns
    }

    pub fn property(&self) -> &PropertyType<'gc> {
        &self.property
    }
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
                        name,
                        ns: vec![ns],
                        property: PropertyType::Property(*property),
                    })
                    .collect(),
            },
        ))
    }

    pub fn add_property(&self, property: PropertyInfo<'gc>) {
        let properties = unsafe { &mut self.0.borrow_mut().properties };

        // Check if property with same name exists
        if let Some(existing) = properties.iter_mut().find(|p| p.name == property.name) {
            // If the namespace is not in the list, add it
            for ns in property.ns {
                if !existing.ns.contains(&ns) {
                    existing.ns.push(ns);
                }
            }
        } else {
            // If no matching property exists, add the new one
            properties.push(property);
        }
    }

    pub fn get_for_multiname(&self, multiname: &Multiname<'gc>) -> Option<PropertyInfo<'gc>> {
        let shape = &self.0.borrow().read().properties;
        for property in shape {
            if property.name == multiname.local_name().expect("multiname has no local name") {
                // check if the namespace set is contained in the property's namespace set
                if property
                    .ns
                    .iter()
                    .any(|ns| multiname.namespace_set().contains(ns))
                {
                    return Some(property.clone());
                }
            }
        }
        None
    }
}
