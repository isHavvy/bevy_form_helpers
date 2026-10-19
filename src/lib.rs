//! UI Form Utilities
//! 
//! A *form* is an entity with the [`FormFields`] component.
//! 
//! Place a [`FieldOf`] relationship on each entity that holds form data
//! relating to the form entity. This will make the field accessible
//! in the form event handlers.
//! 
//! There are three form events, [`Submit`], [`Cancel`], and [`Reset`]. These events
//! are triggered either by activating a [`FormButton`] of the corresponding variant
//! or manually by using [`EntityCommandsFormExt`] and calling `.submit_form()`,
//! `.cancel_form()` or `.reset_form()` on an `EntityCommands` for the form.
//
// There's an example at the very bottom of the file.

use bevy::app::Plugin;
use bevy::ecs::observer::On;
use bevy::ecs::system::{Commands, Query};
use bevy::ecs::{component::Component, entity::Entity, event::EntityEvent,
    system::{EntityCommand, EntityCommands}, template::FromTemplate, 
    world::{EntityWorldMut, World}
};
use bevy::platform::collections::HashMap;
use bevy::ui_widgets::{Activate, Button};

#[derive(Debug)]
pub struct FormPlugin;

impl Plugin for FormPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_observer(on_activate);
    }
}

/// A field of a form.
/// 
/// # Warning
/// 
/// Field names must be unique per form. If not, only one field (at random)
/// will be accessible in the form's event handlers.
#[derive(Debug, Clone, Component, FromTemplate)]
#[relationship(relationship_target = FormFields)]
pub struct FieldOf {
    #[relationship] pub form: Entity,
    pub name: String,
}
// #[DesignQuestion]
// Should this be a String no matter what, or should it be made generic or some other type?
// Pro of it being a String is that we can use the Into<String> machinery.
// Cons is that, well, Strings are way too much memory for what we are doing here.
// But there also should never be e.g. 1000s of forms.
// A generic type also lets people make their own enums for fields.

#[derive(Debug, Component)]
#[relationship_target(relationship = FieldOf)]
pub struct FormFields(Vec<Entity>);

impl FormFields {
    fn to_hash_map(&self, world: &World) -> HashMap<String, Entity> {
        self.0.iter().copied().map(|field_entity| {
            let field_of = world.entity(field_entity).get::<FieldOf>().unwrap();

            (field_of.name.clone(), field_entity)
        })
        .collect()
    }
}

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

#[derive(Debug)]
pub struct SubmitFormCommand;

impl EntityCommand for SubmitFormCommand {
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
pub struct CancelFormCommand;

impl EntityCommand for CancelFormCommand {
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
pub struct ResetFormCommand;

impl EntityCommand for ResetFormCommand {
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
pub trait EntityCommandsFormExt {
    fn submit_form(&mut self);

    fn cancel_form(&mut self);

    fn reset_form(&mut self);
}

impl<'a> EntityCommandsFormExt for EntityCommands<'a> {
    fn submit_form(&mut self) {
        self.queue(SubmitFormCommand);
    }

    fn cancel_form(&mut self) {
        self.queue(CancelFormCommand);
    }

    fn reset_form(&mut self) {
        self.queue(ResetFormCommand);
    }
}

// #[DesignQuestion] Should this be three separate components `SubmitButton`, `CancelButton`, `ResetButton`? 
/// Button that performs a form action
#[derive(Debug, Clone, Copy, Component, FromTemplate)]
#[require(Button)]
pub enum FormButton {
    #[default]
    Submit(Entity),
    Cancel(Entity),
    Reset(Entity),
}

// #[DesignQuestion] This adds a global observer that has to run on every activate.
//                   Should this instead use the lifecycle hook machinery to add/remove an observer when
//                   the FormButton component is added/removed?
//                   It'd be nice if there was a performant way to add a global observer but only on
//                   entitie with specific components.
fn on_activate(
    event: On<Activate>,
    q_form_button: Query<&FormButton>,
    mut commands: Commands,
) {
    let Ok(&button) = q_form_button.get(event.entity) else { return; };

    match button {
        FormButton::Submit(form_entity) => { commands.entity(form_entity).submit_form(); },
        FormButton::Cancel(form_entity) => { commands.entity(form_entity).cancel_form(); },
        FormButton::Reset(form_entity) => { commands.entity(form_entity).reset_form(); },
    }
}

mod example {
    use bevy::prelude::*;
    use super::{Submit, FieldOf, FormButton};

    #[allow(unused, reason = "Example function, to show how the utilities of this module are used.")]
    fn example_form() -> impl Scene {
        use bevy::prelude::*;
        use bevy::ui_widgets::{Button, *};
        use bevy::text::EditableText;

        #[derive(Component, Clone, FromTemplate, Deref)]
        struct FormRef(Entity);

        fn on_submit(
            submit: On<Submit>,
            editable_text_query: Query<&EditableText>,
            checkbox_query: Query<&Checkbox>,
        ) {
            let username = editable_text_query.get(submit.get_field("Name").unwrap()).unwrap();
            let is_playing = checkbox_query.get(submit.get_field("Playing?").unwrap()).unwrap();

            /* do stuff with username and is_playing here */
        }

        bsn! {
            #ExampleForm
            Node {
                flex_direction: FlexDirection::Column,
            }
            on(on_submit)

            Children [
                // Omitting non-field entities for brevity
                FieldOf { form: #ExampleForm, name: { "Name".to_string() } }
                EditableText

                --

                FieldOf { form: #ExampleForm, name: { "Playing?".to_string() }}
                Checkbox

                --

                FormButton::Submit(#ExampleForm)
                Text::new("Submit")
            ]
        }
    }
}