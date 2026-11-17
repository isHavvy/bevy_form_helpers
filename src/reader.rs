//! Built-in field value readers

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::field::ReadField;

#[cfg(feature = "bevy_ui")]
#[derive(Debug, Clone, Copy, Default)]
pub struct IsChecked;

#[cfg(feature = "bevy_ui")]
impl ReadField for IsChecked {
    type Output = bool;

    #[inline]
    fn read_value(&self, entity: EntityRef) -> Option<Self::Output> {
        HasComponent::<bevy::ui::Checked>::new().read_value(entity)
    }
}

#[cfg(feature = "bevy_ui_widgets")]
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadSliderValue;

#[cfg(feature = "bevy_ui_widgets")]
impl ReadField for ReadSliderValue {
    type Output = f32;

    #[inline]
    fn read_value(&self, entity: EntityRef) -> Option<f32> {
        entity.get::<bevy::ui_widgets::SliderValue>().map(|v| v.0)
    }
}

#[cfg(feature = "bevy_text")]
pub use editable_text::{ParseText, ReadEditableText};

#[cfg(feature = "bevy_text")]
mod editable_text {
    use bevy::ecs::world::EntityRef;
    use bevy::text::EditableText;

    use crate::field::ReadField;

    #[derive(Debug, Clone, Copy, Default)]
    pub struct ReadEditableText;

    impl ReadField for ReadEditableText {
        type Output = String;

        #[inline]
        fn read_value(&self, entity: EntityRef) -> Option<String> {
            entity
                .get::<EditableText>()
                .map(|text| text.value().to_string())
        }
    }

    /// Reads the value of a text widget and stores the value if it parses to a specific type.
    #[derive(Debug)]
    pub struct ParseText<T: std::str::FromStr + Clone + Send + Sync + 'static> {
        _marker: std::marker::PhantomData<fn() -> T>,
    }

    impl<T: std::str::FromStr + Clone + Send + Sync + 'static> Clone for ParseText<T> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<T: std::str::FromStr + Clone + Send + Sync + 'static> Copy for ParseText<T> {}

    impl<T: std::str::FromStr + Clone + Send + Sync + 'static> Default for ParseText<T> {
        fn default() -> Self {
            Self {
                _marker: std::marker::PhantomData,
            }
        }
    }

    impl<T: std::str::FromStr + Clone + Send + Sync + 'static> ReadField for ParseText<T> {
        type Output = T;

        #[inline]
        fn read_value(&self, entity: EntityRef) -> Option<T> {
            ReadEditableText.read_value(entity).map(|text| text.parse::<T>().ok()).flatten()
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
    fn read_value(&self, entity: EntityRef) -> Option<C> {
        entity.get::<C>().cloned()
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

    fn read_value(&self, entity: EntityRef) -> Option<bool> {
        Some(entity.get::<C>().is_some())
    }
}

/// Functions implement this trait for one-off functions.
impl<F, T> ReadField for F
where
    F: Clone,
    F: Fn(EntityRef) -> Option<T> + Send + Sync + 'static,
    T: Clone + Send + Sync + 'static,
{
    type Output = T;

    fn read_value(&self, entity: EntityRef) -> Option<T> {
        self(entity)
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
        assert_eq!(reader.read_value(world.entity(e)), Some(Score(9)));

        let empty = world.spawn_empty().id();
        assert_eq!(reader.read_value(world.entity(empty)), None);
    }

    #[test]
    fn has_component_reports_presence() {
        let mut world = World::new();
        let with = world.spawn(Selected).id();
        let without = world.spawn_empty().id();
        let reader = HasComponent::<Selected>::new();
        assert_eq!(reader.read_value(world.entity(with)), Some(true));
        assert_eq!(reader.read_value(world.entity(without)), Some(false));
    }

    #[test]
    fn closure_reader() {
        let mut world = World::new();
        let e = world.spawn(Score(5)).id();
        let reader = |entity: EntityRef| entity.get::<Score>().map(|s| s.0 * 2);
        assert_eq!(reader.read_value(world.entity(e)), Some(10));
    }

    #[test]
    fn defaults_match_new() {
        // Default and new should be interchangeable for the phantom readers.
        let mut world = World::new();
        let e = world.spawn(Score(1)).id();
        assert_eq!(
            ReadComponent::<Score>::default().read_value(world.entity(e)),
            ReadComponent::<Score>::new().read_value(world.entity(e)),
        );
    }

    #[cfg(feature = "bevy_ui")]
    #[test]
    fn is_checked() {
        let mut world = World::new();
        let checked = world.spawn(bevy::ui::Checked).id();
        let unchecked = world.spawn_empty().id();
        assert_eq!(IsChecked.read_value(world.entity(checked)), Some(true));
        assert_eq!(IsChecked.read_value(world.entity(unchecked)), Some(false));
    }

    #[cfg(feature = "bevy_ui_widgets")]
    #[test]
    fn slider_value() {
        let mut world = World::new();
        let e = world.spawn(bevy::ui_widgets::SliderValue(0.75)).id();
        assert_eq!(ReadSliderValue.read_value(world.entity(e)), Some(0.75));

        let empty = world.spawn_empty().id();
        assert_eq!(ReadSliderValue.read_value(world.entity(empty)), None);
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
                ReadEditableText.read_value(world.entity(e)),
                Some("hello".to_string()),
            );

            let empty = world.spawn_empty().id();
            assert_eq!(ReadEditableText.read_value(world.entity(empty)), None);
        }

        #[test]
        fn parse_text_valid() {
            let mut world = World::new();
            let e = world.spawn(EditableText::new("42")).id();
            assert_eq!(ParseText::<i32>::default().read_value(world.entity(e)), Some(42));
        }

        #[test]
        fn parse_text_invalid_is_none() {
            let mut world = World::new();
            let e = world.spawn(EditableText::new("abc")).id();
            assert_eq!(ParseText::<i32>::default().read_value(world.entity(e)), None);
        }

        #[test]
        fn parse_text_missing_component_is_none() {
            let mut world = World::new();
            let e = world.spawn_empty().id();
            assert_eq!(ParseText::<i32>::default().read_value(world.entity(e)), None);
        }

        #[test]
        fn parse_text_float() {
            let mut world = World::new();
            let e = world.spawn(EditableText::new("0.5")).id();
            assert_eq!(ParseText::<f32>::default().read_value(world.entity(e)), Some(0.5));
        }
    }
}