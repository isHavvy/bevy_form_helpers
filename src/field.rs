use std::any::Any;
use std::borrow::Cow;

use bevy::prelude::*;
use bevy::platform::collections::HashMap;

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
    pub name: Cow<'static, str>,
}

#[derive(Debug, Component)]
#[relationship_target(relationship = FieldOf)]
pub struct FormFields(Vec<Entity>);

impl FormFields {
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.0.iter().copied()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> IntoIterator for &'a FormFields {
    type Item = Entity;
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, Entity>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

pub type FieldEntities = HashMap<Cow<'static, str>, Entity>;

pub trait ReadField: Clone + Send + Sync + 'static {
    type Output: Clone + Send + Sync + 'static;

    fn read_value(&self, entity: EntityRef) -> Option<Self::Output>;
}

/// Type erased version of ReadField.
/// 
/// All types that implement ReadField automatically implement this trait.
pub(crate) trait ReadFieldAny: Send + Sync + 'static {
    fn read_value_any(&self, entity: EntityRef) -> Option<AnyFieldValue>;
}

impl<G: ReadField> ReadFieldAny for G {
    fn read_value_any(&self, entity: EntityRef) -> Option<AnyFieldValue> {
        ReadField::read_value(self, entity).map(AnyFieldValue::new)
    }
}

impl AnyFieldValue {
    pub(crate) fn new<T: Any + Send + Sync>(value: T) -> Self {
        Self {
            value: Box::new(value),

            #[cfg(debug_assertions)]
            type_name: std::any::type_name::<T>(),
        }
    }
}

/// Component describing how to read the value of a field.
/// 
/// When used in scenes, use the [`field_reader`] function.
#[derive(Component)]
pub struct FieldReader(Box<dyn ReadFieldAny>);

impl std::fmt::Debug for FieldReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FieldReader(..)")
    }
}

impl FieldReader {
    pub fn new<R: ReadField>(get_field_value: R) -> Self {
        Self(Box::new(get_field_value))
    }
}

impl ReadFieldAny for FieldReader {
    fn read_value_any(&self, entity: EntityRef) -> Option<AnyFieldValue> {
        self.0.read_value_any(entity)
    }
}

impl<R: ReadField> From<R> for FieldReader {
    fn from(reader: R) -> Self {
        FieldReader::new(reader)
    }
}

#[cfg(feature = "bevy_scene")]
pub use field_reader_template::{FieldReaderTemplate, field_reader};

#[cfg(feature = "bevy_scene")]
mod field_reader_template {
    use super::{FieldReader, ReadField};
    use bevy::ecs::template::Template;
    use bevy::scene::{ResolveContext, ResolveSceneError, ResolvedScene, Scene};
    use bevy::ecs::error::Result;

    #[derive(Debug, Clone)]
    pub struct FieldReaderTemplate<R: ReadField>(pub R);

    // Specialization hack.
    impl<R: ReadField> Unpin for FieldReaderTemplate<R> where for<'a> [()]: bevy::ecs::template::SpecializeFromTemplate {}

    impl<R: ReadField> Template for FieldReaderTemplate<R> {
        type Output = FieldReader;

        fn build_template(&self, _context: &mut bevy::ecs::template::TemplateContext) -> Result<Self::Output> {
            Ok(FieldReader::new(self.0.clone()))
        }

        fn clone_template(&self) -> Self {
            self.clone()
        }
    }

    impl<R: ReadField> Scene for FieldReaderTemplate<R> {
        fn resolve(self, _context: &mut ResolveContext, scene: &mut ResolvedScene) -> Result<(), ResolveSceneError> {
            scene.push_template(self);
            Ok(())
        }
    }

    #[must_use]
    pub fn field_reader<R: ReadField>(reader: R) -> FieldReaderTemplate<R> {
        FieldReaderTemplate(reader)
    }
}

// Owned typed erased value
#[derive(Debug)]
pub(crate) struct AnyFieldValue {
    value: Box<dyn Any + Send + Sync>,

    #[cfg(debug_assertions)]
    type_name: &'static str
}

/// A map of field names to their values.
/// 
/// You will usually not need to deal with this type directly.
/// Instead, you can call `Submit::value` which corresponds to this types `get` method.
/// 
/// # Example
/// 
/// ```rust
/// fn on_submit(submit: On<Submit>) {
///     let values: &FieldValues = &submit.values;
/// 
///     let username = values.get::<String>("username");
///     // do stuff with username
/// }
/// ```
#[derive(Debug, Default)]
pub struct FieldValues(pub(crate) HashMap<Cow<'static, str>, AnyFieldValue>);

impl FieldValues {
    pub fn get<T: Clone + 'static>(&self, name: &str) -> Option<T> {
        self.0.get(name)?.value.downcast_ref::<T>().cloned()
    }

    pub fn try_get<T: Clone + 'static>(&self, name: &str) -> Result<Option<T>, FieldValuesGetError> {
        match self.0.get(name) {
            Some(any_value) => {
                match any_value.value.downcast_ref().cloned() {
                    Some(value) => Ok(value),
                    None => Err(FieldValuesGetError::TypeError { #[cfg(debug_assertions)] actual: any_value.type_name }),
                }
            },
            None => Err(FieldValuesGetError::NoField(name.to_string())),
        }
    }

    pub fn get_or<T: Clone + 'static>(&self, name: &str, default: T) -> T {
        self.get(name).unwrap_or(default)
    }

    pub fn contains_field(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(|k| k.as_ref())
    }
}

#[derive(Debug, Clone)]
pub enum FieldValuesGetError {
    NoField(String),
    TypeError {
        #[cfg(debug_assertions)]
        actual: &'static str,
    }
}

impl std::fmt::Display for FieldValuesGetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValuesGetError::NoField(field) => write!(f, "form has no field named {field}"),
            FieldValuesGetError::TypeError { #[cfg(debug_assertions)] actual } => {
                write!(f, "field type is not the requested type")?;
                
                #[cfg(debug_assertions)]
                write!(f, " (actual type: {actual})")?;
                Ok(())
            },
        }
    }
}

impl std::error::Error for FieldValuesGetError {}