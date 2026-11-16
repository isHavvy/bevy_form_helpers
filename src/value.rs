use std::any::Any;

use bevy::{platform::collections::HashMap, prelude::*};

pub trait GetFieldValue: Send + Sync + 'static {
    type Output: Clone + Send + Sync + 'static;

    fn get_value(&self, entity: EntityRef) -> Option<Self::Output>;
}

pub(crate) trait GetFieldValueAny: Send + Sync + 'static {
    fn get_any(&self, entity: EntityRef) -> Option<AnyFieldValue>;
}

impl<G: GetFieldValue> GetFieldValueAny for G {
    fn get_any(&self, entity: EntityRef) -> Option<AnyFieldValue> {
        GetFieldValue::get_value(self, entity).map(AnyFieldValue::new)
    }
}

// This struct exists for debug diagnostics.
#[derive(Debug)]
pub(crate) struct AnyFieldValue {
    value: Box<dyn Any + Send + Sync>,

    #[cfg(debug_assertions)]
    type_name: &'static str
}

impl AnyFieldValue {
    pub(crate) fn new<T: Any + Send + Sync>(value: T) -> Self {
        Self {
            value: Box::new(value),

            #[cfg(debug_assertions)]
            type_name: std::any::type_name::<T>(),
        }
    }
}

#[derive(Component)]
pub struct FieldValue(Box<dyn GetFieldValueAny>);

impl FieldValue {
    pub fn new<G: GetFieldValue>(get_field_value: G) -> Self {
        Self(Box::new(get_field_value))
    }
}

impl GetFieldValueAny for FieldValue {
    fn get_any(&self, entity: EntityRef) -> Option<AnyFieldValue> {
        self.0.get_any(entity)
    }
}

#[derive(Debug, Default)]
pub struct FieldValues(pub(crate) HashMap<String, AnyFieldValue>);

impl FieldValues {
    pub fn get<T: Clone + 'static>(&self, name: &str) -> Option<T> {
        self.0.get(name)?.value.downcast_ref::<T>().cloned()
    }
}