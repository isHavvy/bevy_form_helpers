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
//! or manually by using [`actions::FormActions`] and calling `.submit_form()`,
//! `.cancel_form()` or `.reset_form()` on an `EntityCommands`/`EntityWorldMut` for the form.
//
// There's an example at the very bottom of the file.

use bevy::app::Plugin;
use bevy::ecs::{component::Component, entity::Entity,
    template::FromTemplate, 
    world::World
};
use bevy::platform::collections::HashMap;

pub mod action;
pub mod button;
pub mod event;
pub mod value;

#[derive(Debug)]
pub struct FormPlugin;

impl Plugin for FormPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_observer(button::on_form_button_activate);
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

#[derive(Debug, Component)]
#[relationship_target(relationship = FieldOf)]
pub struct FormFields(Vec<Entity>);

pub type FieldEntities = HashMap<String, Entity>;

mod example {
    use bevy::prelude::*;
    use super::{event::Submit, FieldOf, button::FormButton};

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