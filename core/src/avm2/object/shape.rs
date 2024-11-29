use gc_arena::{Collect, GcCell, Mutation};
use std::borrow::Borrow;

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
/// It stores the property information of an object, for example
/// an object with a field 'x' will have field 'x' in its local storage,
/// which is the slot. The shape will store the property name and the slot id.
/// For a property lookup, it inherently looks up the slot id and read/write
/// to the local storage.
#[derive(Debug, Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct Shape<'gc>(GcCell<'gc, ShapeData<'gc>>);

impl PartialEq for Shape<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.borrow().read().properties == other.0.borrow().read().properties
    }
}

#[derive(Debug, Clone, Collect, PartialEq)]
#[collect(no_drop)]
pub struct ShapeData<'gc> {
    /// Properties in this shape, keyed by local name, value is the slot id.
    pub(crate) properties: Vec<PropertyInfo<'gc>>,
}

#[derive(Debug, Default, Clone, Collect)]
#[collect(no_drop)]
pub struct ShapeManager<'gc> {
    pub(crate) shapes: Vec<Shape<'gc>>,
    pub(crate) next_id: usize,
}

impl<'gc> ShapeManager<'gc> {
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            next_id: 0,
        }
    }

    /// Returns the shape id
    #[inline(always)]
    pub fn get_shape_id(&mut self, mc: &Mutation<'gc>, vtable: &VTable<'gc>) -> usize {
        let shape = Shape::new(mc, vtable);
        // find if the shape already exists
        if let Some((index, _)) = self.shapes.iter().enumerate().find(|(_, s)| **s == shape) {
            return index;
        }

        // if the shape does not exist, add it
        self.add_shape(shape)
    }

    /// Add the new property to the shape, if the property exists, we will return the
    /// same shape id; otherwise, we will create a new shape and get the new shape id.
    #[inline(always)]
    pub fn add_property(
        &mut self,
        mc: &Mutation<'gc>,
        shape_id: usize,
        property: PropertyInfo<'gc>,
    ) -> usize {
        {
            let shape = &self.shapes[shape_id];
            // check if there is any property that is the same
            let properties = unsafe { &mut shape.0.borrow_mut().properties };
            if properties.iter().any(|p| p.name == property.name) {
                // this add_property is a noop, we directly returns the old id
                return shape_id;
            }
        }

        // if no matching property exists, add the new one
        let mut new_properties = self.shapes[shape_id].0.borrow().read().properties.clone();
        new_properties.push(property);
        let new_shape = Shape::new_with_properties(mc, new_properties);

        self.add_shape(new_shape)
    }

    #[inline(always)]
    pub fn get_for_multiname(
        &self,
        shape_id: usize,
        multiname: &Multiname<'gc>,
    ) -> Option<PropertyInfo<'gc>> {
        let shape = &self.shapes[shape_id];
        let properties = &shape.0.borrow().read().properties;
        for property in properties {
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

    fn add_shape(&mut self, shape: Shape<'gc>) -> usize {
        self.shapes.push(shape);
        self.next_id += 1;

        self.next_id - 1
    }
}

impl<'gc> Shape<'gc> {
    fn new(mc: &Mutation<'gc>, vtable: &VTable<'gc>) -> Self {
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

    fn new_with_properties(mc: &Mutation<'gc>, properties: Vec<PropertyInfo<'gc>>) -> Self {
        Self(GcCell::new(mc, ShapeData { properties }))
    }
}
