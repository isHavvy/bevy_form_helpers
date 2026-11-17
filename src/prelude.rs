//! Yep! It's a prelude.
//! 
//! This prelude also pulls in the FormActions trait adding methods to EntityCommands and WorldEntityMut.

pub use crate::action::FormActions;
#[cfg(feature = "bevy_ui_widgets")] pub use crate::button::FormButton;
pub use crate::event::{Cancel, Reset, Submit};
pub use crate::field::{
    FieldOf,
    FieldReader,
    FieldValues,
    FormFields,
    ReadField,
    FieldValuesGetError
};
#[cfg(feature = "bevy_scene")] pub use crate::field::field_reader;
pub use crate::FormPlugin;