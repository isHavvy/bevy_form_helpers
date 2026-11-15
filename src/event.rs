use bevy::prelude::*;
use bevy::platform::collections::HashMap;

#[derive(Debug, EntityEvent)]
pub struct Submit {
    pub entity: Entity,
    pub fields: HashMap<String, Entity>
}

impl Submit {
    pub fn get_field(&self, field: impl Into<String>) -> Option<Entity> {
        self.fields.get(&field.into()).copied()
    }
}

#[derive(Debug, EntityEvent)]
pub struct Cancel {
    pub entity: Entity,
    pub fields: HashMap<String, Entity>
}

impl Cancel {
    pub fn get_field(&self, field: impl Into<String>) -> Option<Entity> {
        self.fields.get(&field.into()).copied()
    }
}

#[derive(Debug, EntityEvent)]
pub struct Reset {
    pub entity: Entity,
    pub fields: HashMap<String, Entity>
}

impl Reset {
    pub fn get_field(&self, field: impl Into<String>) -> Option<Entity> {
        self.fields.get(&field.into()).copied()
    }
}