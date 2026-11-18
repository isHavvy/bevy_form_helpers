use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use crate::action::FormActions as _;

/// A [`Button`] that when [activated](`Activate`), automatically performs a form action.
/// 
/// The entity that each variant holds must point to the form entity that you want the
/// action to be performed on.
#[derive(Debug, Clone, Copy, Component, FromTemplate)]
#[require(Button)]
pub enum FormButton {
    #[default]
    Submit(Entity),
    Cancel(Entity),
    Reset(Entity),
}

impl FormButton {
    pub fn form(&self) -> Entity {
        match self {
            Self::Submit(e) | Self::Cancel(e) | Self::Reset(e) => *e
        }
    }
}

pub(crate) fn on_form_button_activate(
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