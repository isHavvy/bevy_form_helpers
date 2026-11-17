use std::borrow::Cow;

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

/// Inserts a field into FieldEntities, warning about duplicate names in debug builds.
/// This is a raw function because FieldEntities is a type alias.
fn insert_field(
    entities: &mut FieldEntities,
    name: Cow<'static, str>,
    field_entity: Entity,
    form_entity: Entity,
) {
    if let Some(previous) = entities.insert(name.clone(), field_entity) {
        #[cfg(debug_assertions)]
        if previous != field_entity {;
            bevy::log::warn!(
                "Form {form_entity:?} has multiple fields named {name:?}: \
                {previous:?} is being shadowed by {field_entity:?}. \
                Only {field_entity:?} will be accessible in form event handlers."
            );
        }
    }
}

fn collect_entities(world: &World, form_entity: Entity) -> FieldEntities {
    let Some(form_fields) = world.get::<FormFields>(form_entity) else {
        return HashMap::new();
    };

    let mut entities = HashMap::with_capacity(form_fields.len());
    for field_entity in form_fields.iter() {
        if let Some(field_of) = world.get::<FieldOf>(field_entity) {
            insert_field(&mut entities, field_of.name.clone(), field_entity, form_entity);
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

        insert_field(&mut entities, field_of.name.clone(), field_entity, form_entity);

        if let Some(getter) = entity_ref.get::<FieldReader>()
            && let Some(val) = getter.read_value_any(entity_ref)
        {
            values.insert(field_of.name.clone(), val);
        }
    }

    (entities, FieldValues(values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::ReadComponent;

    #[derive(Component, Clone, Debug, PartialEq)]
    struct Score(u32);
    #[derive(Resource, Default)]
    struct Fired(bool);

    fn assert_submit(submit: On<Submit>, mut fired: ResMut<Fired>) {
        assert_eq!(submit.values.get::<Score>("score"), Some(Score(7)));
        assert!(submit.get_field("score").is_some());
        assert!(submit.get_field("absent").is_none());
        fired.0 = true;
    }

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Fired>();
        world.add_observer(assert_submit);
        world
    }

    fn spawn_form(world: &mut World) -> Entity {
        let form = world.spawn_empty().id();
        world.spawn((
            FieldOf { form, name: "score".into() },
            Score(7),
            FieldReader::new(ReadComponent::<Score>::new()),
        ));
        form
    }

    #[test]
    fn submit_collects_values_and_fires() {
        let mut world = setup_world();
        let form = spawn_form(&mut world);
        world.entity_mut(form).submit_form();
        assert!(world.resource::<Fired>().0);
    }

    #[test]
    fn field_without_reader_appears_in_fields_not_values() {
        let mut world = World::new();
        world.add_observer(|submit: On<Submit>| {
            assert!(submit.get_field("bare").is_some());
            assert!(!submit.values.contains_field("bare"));
        });
        let form = world.spawn_empty().id();
        world.spawn((FieldOf { form, name: "bare".into() }, Score(1)));
        world.entity_mut(form).submit_form();
    }

    #[test]
    fn reader_returning_none_omits_value() {
        let mut world = World::new();
        world.add_observer(|submit: On<Submit>| {
            assert!(submit.get_field("missing").is_some());
            assert!(!submit.values.contains_field("missing"));
        });
        let form = world.spawn_empty().id();
        // entity has FieldOf + reader, but NOT the component the reader reads
        world.spawn((
            FieldOf { form, name: "missing".into() },
            FieldReader::new(ReadComponent::<Score>::new()),
        ));
        world.entity_mut(form).submit_form();
    }

    #[test]
    fn form_without_fields_submits_empty() {
        let mut world = World::new();
        world.add_observer(|submit: On<Submit>| {
            assert!(submit.fields.is_empty());
            assert!(submit.values.is_empty());
        });
        let form = world.spawn_empty().id();
        world.entity_mut(form).submit_form();
    }

    #[test]
    fn cancel_and_reset_fire_with_fields() {
        let mut world = World::new();
        world.add_observer(|cancel: On<Cancel>| {
            assert!(cancel.get_field("score").is_some());
        });
        world.add_observer(|reset: On<Reset>| {
            assert!(reset.get_field("score").is_some());
        });
        let form = spawn_form(&mut world);
        world.entity_mut(form).cancel_form();
        world.entity_mut(form).reset_form();
    }
}