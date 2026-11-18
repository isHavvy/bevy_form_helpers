//! A minimal form: a text input, a checkbox, and a submit button.
//!
//! Run with: cargo run --example ui_form
//! 
//! Not that running works at the moment.

use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui_widgets::{Checkbox, TextInput, checkbox_self_update};
use bevy_form::prelude::*;
use bevy_form::field::readers::{ReadEditableText, IsChecked};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FormPlugin))
        .add_systems(Startup, setup.spawn())
        .run();
}

fn setup() -> impl SceneList {
    bsn_list! {
        Camera2d
        --
        @form()
    }
}

fn form() -> impl Scene {
    bsn! {
        #ExampleForm
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(12),
        }
        on(on_submit)
        Children [

            // Text field
            FieldOf { form: #ExampleForm, name: "Name" }
            field_reader(ReadEditableText)
            TextInput
            EditableText { visible_width: { Some(12.) }, allow_newlines: false }
            Node { border: { px(2).all() }, padding: { px(4).all() } }
            BorderColor::from(Color::WHITE)
            TextFont { font_size: FontSize::Px(20.) }

            --

            // Checkbox field
            FieldOf { form: #ExampleForm, name: "Playing?" }
            field_reader(IsChecked)
            Checkbox
            on(checkbox_self_update)
            Node {
                width: px(20),
                height: px(20),
                border: { px(2).all() },
            }
            BorderColor::from(Color::WHITE)
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2))

            --

            // Submit button
            FormButton::Submit(#ExampleForm)
            Node {
                padding: { px(8).all() },
                border: { px(2).all() },
            }
            BorderColor::from(Color::WHITE)
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15))
            Children [
                Text::new("Add Person")
                TextFont { font_size: FontSize::Px(20.) }
            ]
        ]
    }
}

fn on_submit(submit: On<Submit>) {
    let name = submit.values.get::<String>("Name").unwrap();
    let playing = submit.values.get::<bool>("Playing?").unwrap();
    info!("Submitted: name={name:?}, playing={playing}");
}