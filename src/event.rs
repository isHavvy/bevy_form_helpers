use bevy::prelude::*;

use crate::field::FieldEntities;
use crate::field::FieldValues;

#[derive(Debug, EntityEvent)]
pub struct Submit {
    /// The form entity this event was triggered on.
    pub entity: Entity,
    pub fields: FieldEntities,
    pub values: FieldValues,
}

impl Submit {
    pub fn get_field(&self, field: &str) -> Option<Entity> {
        self.fields.get(field).copied()
    }

    pub fn has_field(&self, field: &str) -> bool {
        self.fields.contains_key(field)
    }

    pub fn value<T: Clone + 'static>(&self, name: &str) -> Option<T> {
        self.values.get(name)
    }

    pub fn value_or<T: Clone + 'static>(&self, name: &str, default: T) -> T {
        self.values.get_or(name, default)
    }

    pub fn value_or_default<T: Clone + Default + 'static>(&self, name: &str) -> T {
        self.values.get_or_default(name)
    }
}

#[derive(Debug, EntityEvent)]
pub struct Cancel {
    /// The form entity this event was triggered on.
    pub entity: Entity,
    pub fields: FieldEntities
}

impl Cancel {
    pub fn get_field(&self, field: &str) -> Option<Entity> {
        self.fields.get(field).copied()
    }
}

#[derive(Debug, EntityEvent)]
pub struct Reset {
    /// The form entity this event was triggered on.
    pub entity: Entity,
    pub fields: FieldEntities
}

impl Reset {
    pub fn get_field(&self, field: &str) -> Option<Entity> {
        self.fields.get(field).copied()
    }
}