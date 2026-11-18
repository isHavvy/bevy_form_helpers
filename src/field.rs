use std::any::Any;
use std::borrow::Cow;

use bevy::prelude::*;
use bevy::platform::collections::HashMap;

#[path = "readers.rs"]
pub mod readers;

/// A field of a form.
/// 
/// # Warning
/// 
/// Field names must be unique per form. If not, only one field (unspecified which one)
/// will be accessible in the form's event handlers.
#[derive(Debug, Clone, Component, FromTemplate)]
#[relationship(relationship_target = FormFields)]
pub struct FieldOf {
    #[relationship] pub form: Entity,
    pub name: Cow<'static, str>,
}

/// List of fields that have this entity as its form.
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

/// A type that can read an owned value from an [`Entity`] + [`World`].
pub trait ReadField: Clone + Send + Sync + 'static {
    type Output: Clone + Send + Sync + 'static;

    fn read_value(&self, entity: Entity, world: &World) -> Option<Self::Output>;
}

/// Type erased version of ReadField.
/// 
/// All types that implement ReadField automatically implement this trait.
/// 
/// This is stored in the [`FieldReader`] component since it does not have
/// any generic type parameters. 
pub(crate) trait ReadFieldAny: Send + Sync + 'static {
    fn read_value_any(&self, entity: Entity, world: &World) -> Option<AnyFieldValue>;
}

impl<R: ReadField> ReadFieldAny for R {
    fn read_value_any(&self, entity: Entity, world: &World) -> Option<AnyFieldValue> {
        ReadField::read_value(self, entity, world).map(AnyFieldValue::new)
    }
}

/// Component describing how to read the value of a field.
/// 
/// When used in scenes, use the [`field_reader`] function.
/// 
/// You can pass in one of the provided readers from the [`readers`] module
/// or a function `fn(Entity, &World) -> Option<YourDataType>` or make a type
/// that implements [`ReadField`].
/// 
/// `FieldReader::new(|e| e.get::<Name>().clone())`
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
    fn read_value_any(&self, entity: Entity, world: &World) -> Option<AnyFieldValue> {
        self.0.read_value_any(entity, world)
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

impl AnyFieldValue {
    pub(crate) fn new<T: Any + Send + Sync>(value: T) -> Self {
        Self {
            value: Box::new(value),

            #[cfg(debug_assertions)]
            type_name: std::any::type_name::<T>(),
        }
    }
}

/// A map of field names to their values.
/// 
/// Note that only fields that have a [`FieldReader`] component will provide values.
/// 
/// You will usually not need to deal with this type directly.
/// Instead, you can call [`Submit::value`](`crate::event::Submit::value`) which corresponds
/// to this type's [`get`](Self::get) method.
/// 
/// # Example
/// 
/// ```rust,ignore
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

    pub fn try_get<T: Clone + 'static>(&self, name: &str) -> Result<T, FieldValuesGetError> {
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

    /// Get the value of the field. If it doesn't exist, return the passed in value instead.
    pub fn get_or<T: Clone + 'static>(&self, name: &str, default: T) -> T {
        self.get(name).unwrap_or(default)
    }

    /// Get the value of the field. If it doesn't exist, return the default value for the type instead.
    pub fn get_or_default<T: Clone + Default + 'static>(&self, name: &str) -> T {
        self.get(name).unwrap_or_default()
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

    /// Provides an iterator of the field names that have values.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(|k| k.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn single_value<T: Any + Send + Sync>(name: &'static str, value: T) -> FieldValues {
        let mut map = HashMap::new();
        map.insert(Cow::Borrowed(name), AnyFieldValue::new(value));
        FieldValues(map)
    }

    // ---- FieldValues::get ----

    #[test]
    fn get_returns_typed_value() {
        let values = single_value("score", 42u32);
        assert_eq!(values.get::<u32>("score"), Some(42));
    }

    #[test]
    fn get_wrong_type_is_none() {
        let values = single_value("score", 42u32);
        assert_eq!(values.get::<i32>("score"), None);
        assert_eq!(values.get::<String>("score"), None);
    }

    #[test]
    fn get_missing_field_is_none() {
        let values = single_value("score", 42u32);
        assert_eq!(values.get::<u32>("nope"), None);
    }

    #[test]
    fn get_or_falls_back() {
        let values = single_value("score", 42u32);
        assert_eq!(values.get_or("score", 0u32), 42);
        assert_eq!(values.get_or("missing", 7u32), 7);
        assert_eq!(values.get_or("score", 7i32), 7); // wrong type also falls back
    }

    // ---- try_get ----

    #[test]
    fn try_get_ok() {
        let values = single_value("score", 42u32);
        assert_eq!(values.try_get::<u32>("score").unwrap(), 42);
    }

    #[test]
    fn try_get_no_field() {
        let values = single_value("score", 42u32);
        match values.try_get::<u32>("missing") {
            Err(FieldValuesGetError::NoField(name)) => assert_eq!(name, "missing"),
            other => panic!("expected NoField, got {other:?}"),
        }
    }

    #[test]
    fn try_get_type_error() {
        let values = single_value("score", 42u32);
        let err = values.try_get::<String>("score").unwrap_err();
        assert!(matches!(err, FieldValuesGetError::TypeError { .. }));

        #[cfg(debug_assertions)]
        if let FieldValuesGetError::TypeError { actual } = err {
            assert_eq!(actual, std::any::type_name::<u32>());
        }
    }

    // ---- map surface ----

    #[test]
    fn contains_len_names() {
        let mut map = HashMap::new();
        map.insert(Cow::Borrowed("a"), AnyFieldValue::new(1u32));
        map.insert(Cow::Borrowed("b"), AnyFieldValue::new(String::from("x")));
        let values = FieldValues(map);

        assert!(values.contains_field("a"));
        assert!(!values.contains_field("c"));
        assert_eq!(values.len(), 2);
        assert!(!values.is_empty());
        assert!(FieldValues::default().is_empty());

        let mut names: Vec<_> = values.names().collect();
        names.sort_unstable();
        assert_eq!(names, ["a", "b"]);
    }

    #[test]
    fn owned_cow_keys_work() {
        let mut map = HashMap::new();
        map.insert(Cow::Owned(format!("field_{}", 1)), AnyFieldValue::new(true));
        let values = FieldValues(map);
        // lookup by &str must work regardless of Borrowed/Owned key
        assert_eq!(values.get::<bool>("field_1"), Some(true));
    }
}