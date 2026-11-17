use bevy::prelude::*;
use bevy::platform::collections::HashMap;

use crate::field::{FieldReader, FieldValues, ReadFieldAny as _};
use crate::field::{FieldEntities, FieldOf, FormFields};
use crate::event::{Cancel, Reset, Submit};

#[derive(Debug)]
pub struct SubmitForm;

impl EntityCommand for SubmitForm {
    type Out = ();

    fn apply(self, mut entity: EntityWorldMut) -> Self::Out {
        let (form_entity, world) = (entity.id(), entity.world());
        let (fields, values) = collect_entities_and_values(world, form_entity);
        entity.trigger(|form_entity| Submit { entity: form_entity, fields, values });
    }
}

#[derive(Debug)]
pub struct CancelForm;

impl EntityCommand for CancelForm {
    type Out = ();

    fn apply(self, mut entity: EntityWorldMut) -> Self::Out {
        let (form_entity, world) = (entity.id(), entity.world());
        let fields = collect_entities(world, form_entity);
        entity.trigger(|form_entity| Cancel { entity: form_entity, fields,  });
    }
}

#[derive(Debug)]
pub struct ResetForm;

impl EntityCommand for ResetForm {
    type Out = ();

    fn apply(self, mut entity: EntityWorldMut) -> Self::Out {
        let (form_entity, world) = (entity.id(), entity.world());
        let fields = collect_entities(world, form_entity);
        entity.trigger(|form_entity| Reset { entity: form_entity, fields,  });
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

fn collect_entities(world: &World, form_entity: Entity) -> FieldEntities {
    let Some(form_fields) = world.get::<FormFields>(form_entity) else {
        return HashMap::new();
    };

    let mut entities = HashMap::with_capacity(form_fields.len());
    for field_entity in form_fields.iter() {
        if let Some(field_of) = world.get::<FieldOf>(field_entity) {
            entities.insert(field_of.name.clone(), field_entity);
        }
    }
    entities
}

fn collect_entities_and_values(
    world: &World,
    form_entity: Entity,
) -> (FieldEntities, FieldValues) {
    let Some(form_fields) = world.get::<FormFields>(form_entity) else {
        return (HashMap::new(), FieldValues::default());
    };

    let mut entities = HashMap::with_capacity(form_fields.len());
    let mut values = HashMap::new();

    for field_entity in form_fields.iter() {
        let entity_ref = world.entity(field_entity);
        let Some(field_of) = entity_ref.get::<FieldOf>() else {
            continue;
        };

        entities.insert(field_of.name.clone(), field_entity);

        if let Some(getter) = entity_ref.get::<FieldReader>()
            && let Some(val) = getter.read_value_any(entity_ref)
        {
            values.insert(field_of.name.clone(), val);
        }
    }

    (entities, FieldValues(values))
}