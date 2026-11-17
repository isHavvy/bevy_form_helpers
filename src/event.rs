use bevy::prelude::*;

use crate::field::FieldEntities;
use crate::field::FieldValues;

#[derive(Debug, EntityEvent)]
pub struct Submit {
    pub entity: Entity,
    pub fields: FieldEntities,
    pub values: FieldValues,
}

impl Submit {
    pub fn get_field(&self, field: &str) -> Option<Entity> {
        self.fields.get(field).copied()
    }

    pub fn value<T: Clone + 'static>(&self, name: &str) -> Option<T> {
        self.values.get(name)
    }
}

#[derive(Debug, EntityEvent)]
pub struct Cancel {
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
    pub entity: Entity,
    pub fields: FieldEntities
}

impl Reset {
    pub fn get_field(&self, field: &str) -> Option<Entity> {
        self.fields.get(field).copied()
    }
}