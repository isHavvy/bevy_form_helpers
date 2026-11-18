//! Built-in [`ReadField`] implementations for common Bevy widgets.
//!
//! All readers in this module are gated behind their respective features.
//! See the [crate-level docs](crate) for the feature list.

// This module's parent module is field.rs

use std::marker::PhantomData;

use bevy::prelude::*;

use super::ReadField;

#[cfg(feature = "bevy_ui")]
#[derive(Debug, Clone, Copy, Default)]
pub struct IsChecked;

#[cfg(feature = "bevy_ui")]
impl ReadField for IsChecked {
    type Output = bool;

    #[inline]
    fn read_value(&self, entity: Entity, world: &World) -> Option<Self::Output> {
        HasComponent::<bevy::ui::Checked>::new().read_value(entity, world)
    }
}

#[cfg(feature = "bevy_ui_widgets")]
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadSliderValue;

#[cfg(feature = "bevy_ui_widgets")]
impl ReadField for ReadSliderValue {
    type Output = f32;

    #[inline]
    fn read_value(&self, entity: Entity, world: &World) -> Option<f32> {
        world.get::<bevy::ui_widgets::SliderValue>(entity).map(|value| value.0)
    }
}

#[cfg(feature = "bevy_text")]
pub use editable_text::{ParseEditableText, ReadEditableText};

#[cfg(feature = "bevy_text")]
mod editable_text {
    use bevy::prelude::*;
    use bevy::text::EditableText;

    use crate::field::ReadField;

    #[derive(Debug, Clone, Copy, Default)]
    pub struct ReadEditableText;

    impl ReadField for ReadEditableText {
        type Output = String;

        #[inline]
        fn read_value(&self, entity: Entity, world: &World) -> Option<String> {
            world.get::<EditableText>(entity).map(|text| text.value().to_string())
        }
    }

    /// Reads the value of a text widget and stores the value if it parses to a specific type.
    #[derive(Debug)]
    pub struct ParseEditableText<T: std::str::FromStr + Clone + Send + Sync + 'static> {
        pub trim: bool,
        _marker: std::marker::PhantomData<fn() -> T>,
    }

    impl<T: std::str::FromStr + Clone + Send + Sync + 'static> ParseEditableText<T> {
        pub fn new() -> Self {
            Default::default()
        }

        pub fn raw() -> Self {
            Self {
                trim: false,
                ..Default::default()
            }
        }
    }

    impl<T: std::str::FromStr + Clone + Send + Sync + 'static> Clone for ParseEditableText<T> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<T: std::str::FromStr + Clone + Send + Sync + 'static> Copy for ParseEditableText<T> {}

    impl<T: std::str::FromStr + Clone + Send + Sync + 'static> Default for ParseEditableText<T> {
        fn default() -> Self {
            Self {
                trim: true,
                _marker: std::marker::PhantomData,
            }
        }
    }

    impl<T: std::str::FromStr + Clone + Send + Sync + 'static> ReadField for ParseEditableText<T> {
        type Output = T;

        #[inline]
        fn read_value(&self, entity: Entity, world: &World) -> Option<T> {
            let text = ReadEditableText.read_value(entity, world)?;
            let text = if self.trim { text.trim() } else { text.as_str() };
            text.parse::<T>().ok()
        }
    }
}

/// A field reader that reads a specific component off an entity.
#[derive(Debug)]
pub struct ReadComponent<C: Component + Clone + Send + Sync + 'static> {
    _marker: std::marker::PhantomData<fn () -> C>
}

impl<C: Component + Clone + Send + Sync> Clone for ReadComponent<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: Component + Clone + Send + Sync> Copy for ReadComponent<C> {}

impl<C: Component + Clone + Send + Sync> Default for ReadComponent<C> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C: Component + Clone + Send + Sync + 'static> ReadComponent<C> {
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<C: Component + Clone + Send + Sync + 'static> ReadField for ReadComponent<C> {
    type Output = C;

    #[inline]
    fn read_value(&self, entity: Entity, world: &World) -> Option<C> {
        world.get::<C>(entity).cloned()
    }
}

/// The field value is whether a component, e.g. `Checked`, exists on the field entity.
#[derive(Debug)]
pub struct HasComponent<C: Component> {
    _marker: std::marker::PhantomData<fn () -> C>
}

impl<C: Component> Clone for HasComponent<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: Component> Copy for HasComponent<C> {}

impl<C: Component> Default for HasComponent<C> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C: Component> HasComponent<C> {
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<C: Component> ReadField for HasComponent<C> {
    type Output = bool;

    fn read_value(&self, entity: Entity, world: &World) -> Option<bool> {
        Some(world.entity(entity).contains::<C>())
    }
}

#[cfg(feature = "bevy_ui_widgets")]
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadListBoxSelection;

#[cfg(feature = "bevy_ui_widgets")]
impl ReadField for ReadListBoxSelection {
    type Output = Entity;

    fn read_value(&self, entity: Entity, world: &World) -> Option<Entity> {
        let children = world.get::<Children>(entity)?;
        children.iter()
            .find(|&child| world.get::<bevy::ui::Selected>(child).is_some())
    }
}

#[cfg(feature = "bevy_ui_widgets")]
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadListBoxSelections;

#[cfg(feature = "bevy_ui_widgets")]
impl ReadField for ReadListBoxSelections {
    type Output = Vec<Entity>;

    fn read_value(&self, entity: Entity, world: &World) -> Option<Vec<Entity>> {
        let children = world.get::<Children>(entity)?;
        let selected: Vec<Entity> = children.iter()
            .filter(|&child| world.get::<bevy::ui::Selected>(child).is_some())
            .collect();
        Some(selected)
    }
}

#[cfg(feature = "bevy_ui_widgets")]
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadRadioGroupSelection;

#[cfg(feature = "bevy_ui_widgets")]
impl ReadField for ReadRadioGroupSelection {
    type Output = Entity;

    fn read_value(&self, entity: Entity, world: &World) -> Option<Entity> {
        let children = world.get::<Children>(entity)?;
        children.iter()
            .find(|&child| world.get::<bevy::ui::Checked>(child).is_some())
    }
}

/// Functions taking an Entity and &World are field readers.
impl<F, T> ReadField for F
where
    F: Clone,
    F: Fn(Entity, &World) -> Option<T> + Send + Sync + 'static,
    T: Clone + Send + Sync + 'static,
{
    type Output = T;

    fn read_value(&self, entity: Entity, world: &World) -> Option<T> {
        self(entity, world)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component, Clone, Debug, PartialEq)]
    struct Score(u32);

    #[derive(Component)]
    struct Selected;

    // ---- generic readers (always available) ----

    #[test]
    fn read_component_clones() {
        let mut world = World::new();
        let e = world.spawn(Score(9)).id();
        let reader = ReadComponent::<Score>::new();
        assert_eq!(reader.read_value(e, &world), Some(Score(9)));

        let empty = world.spawn_empty().id();
        assert_eq!(reader.read_value(empty, &world), None);
    }

    #[test]
    fn has_component_reports_presence() {
        let mut world = World::new();
        let with = world.spawn(Selected).id();
        let without = world.spawn_empty().id();
        let reader = HasComponent::<Selected>::new();
        assert_eq!(reader.read_value(with, &world), Some(true));
        assert_eq!(reader.read_value(without, &world), Some(false));
    }

    #[test]
    fn closure_reader() {
        let mut world = World::new();
        let e = world.spawn(Score(5)).id();
        let reader = |entity: Entity, world: &World| world.get::<Score>(entity).map(|s| s.0 * 2);
        assert_eq!(reader.read_value(e, &world), Some(10));
    }

    #[test]
    fn defaults_match_new() {
        // Default and new should be interchangeable for the phantom readers.
        let mut world = World::new();
        let e = world.spawn(Score(1)).id();
        assert_eq!(
            ReadComponent::<Score>::default().read_value(e, &world),
            ReadComponent::<Score>::new().read_value(e, &world),
        );
    }

    #[cfg(feature = "bevy_ui")]
    #[test]
    fn is_checked() {
        let mut world = World::new();
        let checked = world.spawn(bevy::ui::Checked).id();
        let unchecked = world.spawn_empty().id();
        assert_eq!(IsChecked.read_value(checked, &world), Some(true));
        assert_eq!(IsChecked.read_value(unchecked, &world), Some(false));
    }

    #[cfg(feature = "bevy_ui_widgets")]
    #[test]
    fn slider_value() {
        let mut world = World::new();
        let e = world.spawn(bevy::ui_widgets::SliderValue(0.75)).id();
        assert_eq!(ReadSliderValue.read_value(e, &world), Some(0.75));

        let empty = world.spawn_empty().id();
        assert_eq!(ReadSliderValue.read_value(empty, &world), None);
    }

    #[cfg(feature = "bevy_text")]
    mod editable_text_tests {
        use super::*;
        use bevy::text::EditableText;

        #[test]
        fn read_editable_text() {
            let mut world = World::new();
            let e = world.spawn(EditableText::new("hello")).id();
            assert_eq!(
                ReadEditableText.read_value(e, &world),
                Some("hello".to_string()),
            );

            let empty = world.spawn_empty().id();
            assert_eq!(ReadEditableText.read_value(empty, &world), None);
        }

        #[test]
        fn parse_text_valid() {
            let mut world = World::new();
            let e = world.spawn(EditableText::new("42")).id();
            assert_eq!(ParseEditableText::<i32>::default().read_value(e, &world), Some(42));
        }

        #[test]
        fn parse_text_invalid_is_none() {
            let mut world = World::new();
            let e = world.spawn(EditableText::new("abc")).id();
            assert_eq!(ParseEditableText::<i32>::default().read_value(e, &world), None);
        }

        #[test]
        fn parse_text_missing_component_is_none() {
            let mut world = World::new();
            let e = world.spawn_empty().id();
            assert_eq!(ParseEditableText::<i32>::default().read_value(e, &world), None);
        }

        #[test]
        fn parse_text_float() {
            let mut world = World::new();
            let e = world.spawn(EditableText::new("0.5")).id();
            assert_eq!(ParseEditableText::<f32>::default().read_value(e, &world), Some(0.5));
        }

        #[test]
        fn parse_text_trims_by_default() {
            let mut world = World::new();
            let e = world.spawn(EditableText::new("  42  \n")).id();
            assert_eq!(ParseEditableText::<i32>::default().read_value(e, &world), Some(42));
        }

        #[test]
        fn parse_text_raw_preserves_whitespace() {
            let mut world = World::new();
            // " 42 " won't parse as i32 untrimmed
            let e = world.spawn(EditableText::new(" 42 ")).id();
            assert_eq!(ParseEditableText::<i32>::raw().read_value(e, &world), None);
            // But a type that accepts spaces would work (contrived example)
            let e2 = world.spawn(EditableText::new("hello")).id();
            assert_eq!(ParseEditableText::<String>::raw().read_value(e2, &world), Some("hello".to_string()));
        }
    }

    #[cfg(feature = "bevy_ui_widgets")]
    mod listbox_tests {
        use super::*;
        use bevy::ui::Selected;

        #[test]
        fn selects_child_with_selected_component() {
            let mut world = World::new();
            let child_a = world.spawn_empty().id();
            let child_b = world.spawn(Selected).id();
            let listbox = world.spawn_empty().id();
            world.entity_mut(listbox).add_children(&[child_a, child_b]);

            assert_eq!(
                ReadListBoxSelection.read_value(listbox, &world),
                Some(child_b),
            );
        }

        #[test]
        fn none_when_no_selected_child() {
            let mut world = World::new();
            let child = world.spawn_empty().id();
            let listbox = world.spawn_empty().id();
            world.entity_mut(listbox).add_children(&[child]);

            assert_eq!(ReadListBoxSelection.read_value(listbox, &world), None);
        }

        #[test]
        fn none_when_no_children() {
            let mut world = World::new();
            let listbox = world.spawn_empty().id();
            assert_eq!(ReadListBoxSelection.read_value(listbox, &world), None);
        }

        #[test]
        fn returns_first_selected_when_multiple() {
            let mut world = World::new();
            let child_a = world.spawn(Selected).id();
            let child_b = world.spawn(Selected).id();
            let listbox = world.spawn_empty().id();
            world.entity_mut(listbox).add_children(&[child_a, child_b]);

            // Single-select semantics: first match wins
            assert_eq!(
                ReadListBoxSelection.read_value(listbox, &world),
                Some(child_a),
            );
        }
    }

    #[cfg(feature = "bevy_ui_widgets")]
    mod radio_tests {
        use super::*;
        use bevy::ui::Checked;

        #[test]
        fn selects_checked_child() {
            let mut world = World::new();
            let a = world.spawn_empty().id();
            let b = world.spawn(Checked).id();
            let group = world.spawn_empty().id();
            world.entity_mut(group).add_children(&[a, b]);
            assert_eq!(ReadRadioGroupSelection.read_value(group, &world), Some(b));
        }

        #[test]
        fn none_when_nothing_checked() {
            let mut world = World::new();
            let a = world.spawn_empty().id();
            let group = world.spawn_empty().id();
            world.entity_mut(group).add_children(&[a]);
            assert_eq!(ReadRadioGroupSelection.read_value(group, &world), None);
        }

        #[test]
        fn none_without_children() {
            let mut world = World::new();
            let group = world.spawn_empty().id();
            assert_eq!(ReadRadioGroupSelection.read_value(group, &world), None);
        }
    }
}