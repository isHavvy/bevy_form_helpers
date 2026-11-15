use bevy::{ecs::{entity::Entity, system::{EntityCommand, EntityCommands}, world::EntityWorldMut}, platform::collections::HashMap};

use crate::FormFields;
use crate::event::{Cancel, Reset, Submit};

#[derive(Debug)]
pub struct SubmitForm;

impl EntityCommand for SubmitForm {
    type Out = ();

    fn apply(self, mut entity: EntityWorldMut) -> Self::Out {
        let fields = entity.get::<FormFields>().unwrap();
        let fields: HashMap<String, Entity> = fields.to_hash_map(entity.world());
        let id = entity.id();
        entity.world_scope(|world| {
            world.trigger(Submit { entity: id, fields });
        });
    }
}

#[derive(Debug)]
pub struct CancelForm;

impl EntityCommand for CancelForm {
    type Out = ();

    fn apply(self, mut entity: EntityWorldMut) -> Self::Out {
        let fields = entity.get::<FormFields>().unwrap();
        let fields: HashMap<String, Entity> = fields.to_hash_map(entity.world());
        let id = entity.id();
        entity.world_scope(|world| {
            world.trigger(Cancel { entity: id, fields });
        });
    }
}

#[derive(Debug)]
pub struct ResetForm;

impl EntityCommand for ResetForm {
    type Out = ();

    fn apply(self, mut entity: EntityWorldMut) -> Self::Out {
        let fields = entity.get::<FormFields>().unwrap();
        let fields: HashMap<String, Entity> = fields.to_hash_map(entity.world());
        let id = entity.id();
        entity.world_scope(|world| {
            world.trigger(Reset { entity: id, fields });
        });
    }
}

/// Form commands extension trait for EntityCommands.
/// 
/// This extension trait provides three methods that will trigger events of the
/// same name on a form entity. Use these commands instead of triggering the
/// events directly so that you don't have to manually create the `fields` field
/// for the events.
pub trait FormActions {
    fn submit_form(&mut self) -> &mut Self;

    fn cancel_form(&mut self) -> &mut Self;

    fn reset_form(&mut self) -> &mut Self;
}

impl<'a> FormActions for EntityCommands<'a> {
    fn submit_form(&mut self) -> &mut Self {
        self.queue(SubmitForm)
    }

    fn cancel_form(&mut self) -> &mut Self {
        self.queue(CancelForm)
    }

    fn reset_form(&mut self) -> &mut Self {
        self.queue(ResetForm)
    }
}

impl<'w> FormActions for EntityWorldMut<'w> {
    fn submit_form(&mut self) -> &mut Self {
        self.reborrow_scope(|entity| SubmitForm.apply(entity));
        self
    }

    fn cancel_form(&mut self) -> &mut Self {
        self.reborrow_scope(|entity| CancelForm.apply(entity));
        self
    }

    fn reset_form(&mut self) -> &mut Self {
        self.reborrow_scope(|entity| ResetForm.apply(entity));
        self
    }
}