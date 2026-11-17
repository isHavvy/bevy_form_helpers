//! Form Utilities
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
//! 
//! Note that a form doesn't have to be a UI form. A puzzle that gets checked
//! when the user performs an in-game action such as talking to an NPC could use this library.
//! 
//! # Features
//! 
//! By default, all features are enabled.
//!
//! - `bevy_scene`: enables [`field::field_reader`] for use in `bsn!` scenes.
//!   Without it, you can insert [`field::FieldReader`] components directly.
//! - `bevy_text`: built-in readers for `EditableText`.
//! - `bevy_ui`: built-in bevy_ui readers.
//! - `bevy_ui_widgets`: [`button::FormButton`] and ui_widgets readers.
//!   Pulls in `bevy_ui`.

use bevy::app::Plugin;

pub mod action;
#[cfg(feature = "bevy_ui_widgets")] pub mod button;
pub mod event;
pub mod field;
pub mod reader;

pub mod prelude;

#[derive(Debug)]
pub struct FormPlugin;

impl Plugin for FormPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        #[cfg(feature = "bevy_ui_widgets")]
        app.add_observer(button::on_form_button_activate);
    }
}