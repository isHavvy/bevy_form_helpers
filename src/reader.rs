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