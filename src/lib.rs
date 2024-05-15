#![doc = include_str!("../README.md")]
/*#![warn(
    clippy::pedantic,
    clippy::doc_markdown,
    clippy::redundant_closure,
    clippy::explicit_iter_loop,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::print_stdout,
    clippy::cast_possible_truncation,
    clippy::unwrap_used,
    clippy::map_unwrap_or,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences,
    rust_2018_idioms,
    rust_2018_compatibility,
    rust_2021_compatibility
)]*/
#![allow(clippy::module_name_repetitions)]
#![allow(private_interfaces)]

use core::fmt;

use serde::de::{DeserializeSeed, Deserializer, MapAccess, Visitor, IntoDeserializer};

mod error;
mod keywords;

pub use error::{SchemaError, ValidationError};
use keywords::{Node, Type};

/*
pub(crate) enum Type {
    Array,
    Boolean,
    Integer,
    Null,
    Number,
    Object,
    String,
}
*/

#[derive(Debug)]
pub enum Schema {
    /// A literal value, not an object
    Type(Type),
    /// An object with the following properties as a tuple of (Key, Schema)
    Properties(Vec<(String, Schema)>),
}

impl Schema {
    #[cfg_attr(not(test), allow(unused))]
    pub(crate) fn new_properties(
        properties: impl Iterator<Item = (impl Into<String>, Schema)>,
    ) -> Self {
        Self::Properties(properties.map(|(k, v)| (k.into(), v)).collect())
    }
}

/// A JSON Schema validator.
pub struct Validator {
    schema: Schema,
}

impl Validator {
    pub fn validate(&self, instance: &str) -> Result<(), ValidationError> {
        match (SerdeValidator { schema: &self.schema, name: "root".into(), depth: 0 }).validate(instance) {
           ValidationResult::Success => Ok(()),
           ValidationResult::Error(err) => Err(err),
        }
    }

    pub fn new(_: &serde_json::Value) -> Result<Validator, SchemaError> {
        // NOTE: Hardcoded for simplicity, as we're not actually optimizing the schema parsing

        let eleven = Schema::new_properties([
           ("inner", Schema::Type(Type::String)),
           ("another", Schema::Type(Type::String)),
        ].into_iter());
        /*let ten = Schema::new_properties([
           ("inner", eleven),
        ].into_iter());
        let nine = Schema::new_properties([
           ("inner", ten),
        ].into_iter());
        let eight = Schema::new_properties([
            ("inner", nine),
        ].into_iter());
        let seven = Schema::new_properties([
            ("inner", eight),
        ].into_iter());
        let six = Schema::new_properties([
            ("inner", seven),
        ].into_iter());
        let five = Schema::new_properties([
            ("inner", six),
        ].into_iter());
        let four = Schema::new_properties([
            ("inner", five),
        ].into_iter());*/
        let three = Schema::new_properties([
            ("inner", eleven),
        ].into_iter());
        let two = Schema::new_properties([
            ("inner", three),
        ].into_iter());
        let one = Schema::new_properties([
            ("inner", two),
        ].into_iter());
        Ok(Validator { schema: one })
    }
}

impl fmt::Debug for Validator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Validator").finish()
    }
}

#[derive(Debug, PartialEq)]
enum ValidationResult {
    Success,
    Error(ValidationError),
}

#[derive(Debug)]
struct SerdeValidator<'a> {
    schema: &'a Schema,
    depth: usize,
    name: &'a str,
}

impl<'a> SerdeValidator<'a> {
    fn validate(&self, input: &str) -> ValidationResult {
        let deserializer = &mut serde_json::Deserializer::from_str(input);
        match deserializer.deserialize_any(ValidatorVisitor {
            schema: self.schema,
            depth: self.depth + 1,
            name: self.name,
        }) {
            Ok(ValidationResult::Success) => ValidationResult::Success,
            Ok(ValidationResult::Error(err)) => ValidationResult::Error(err),
            Err(_serde_err) => {
                let mut stack = Vec::with_capacity(self.depth);
                stack.push(self.name.into());
                let value: serde_json::Value = deserializer.deserialize_any::<serde_json::Value>(deserializer).unwrap();
                let err = ValidationError::new(format!("serde: {_serde_err}"), stack);
                return ValidationResult::Error(err);
            }
        }
    }
}

impl<'de, 'a> DeserializeSeed<'de> for SerdeValidator<'a> {
    type Value = ValidationResult;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match deserializer.deserialize_any(ValidatorVisitor {
            schema: self.schema,
            depth: self.depth + 1,
            name: self.name,
        }) {
            Ok(ValidationResult::Success) => ValidationResult::Success,
            Ok(ValidationResult::Error(err)) => ValidationResult::Error(err),
            Err(_serde_err) => {
                let mut stack = Vec::with_capacity(self.depth);
                stack.push(self.name.into());
                let err = ValidationError::new("Did not find expected type", stack);
                ValidationResult::Error(err)
            }
        })
    }
}

struct ValidatorVisitor<'a> {
    schema: &'a Schema,
    depth: usize,
    name: &'a str,
}

impl<'de, 'a> Visitor<'de> for ValidatorVisitor<'a> {
    type Value = ValidationResult;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a valid JSON object according to the schema")
    }

    // fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    //     where
    //         E: serde::de::Error, {
    //             match self.schema {
    //                 Schema::Type(Type::String) => return Ok(ValidationResult::Success),
    //                 Schema::Type(typ) => {
    //                     let mut stack = Vec::with_capacity(self.depth);
    //                     stack.push(self.name.into());
    //                     let err = ValidationError::new(format!("{v} is not expected type {typ}"), stack);
    //                     return Ok(ValidationResult::Error(err));
    //                 }
    //                 Schema::Properties(_) => {
    //                     let mut stack = Vec::with_capacity(self.depth);
    //                     stack.push(self.name.into());
    //                     let err = ValidationError::new(format!("expected object, found string {v}"), stack);
    //                     return Ok(ValidationResult::Error(err));
    //                 }
    //             }
    // }
    //
    // fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    //     where
    //         E: serde::de::Error, {
    //             match self.schema {
    //                 Schema::Type(Type::Number) => return Ok(ValidationResult::Success),
    //                 Schema::Type(typ) => {
    //                     let mut stack = Vec::with_capacity(self.depth);
    //                     stack.push(self.name.into());
    //                     let err = ValidationError::new(format!("{v} is not expected type {typ}"), stack);
    //                     return Ok(ValidationResult::Error(err));
    //                 }
    //                 Schema::Properties(_) => {
    //                     let mut stack = Vec::with_capacity(self.depth);
    //                     stack.push(self.name.into());
    //                     let err = ValidationError::new(format!("expected object, found number {v}"), stack);
    //                     return Ok(ValidationResult::Error(err));
    //                 }
    //             }
    //
    // }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        match self.schema {
            Schema::Type(_ty) => {
                // Not expecting a map if the schema is a simple type.
                let mut stack = Vec::with_capacity(self.depth);
                stack.push(self.name.into());
                let err = ValidationError::new("Not expecting a map here", stack);
                return Ok(ValidationResult::Error(err));
            }
            Schema::Properties(properties) => {
                for (key, schema) in properties.iter() {
                    match map.next_key::<&str>() {
                        Ok(Some(child_key)) if child_key == key => (), // OK
                        Ok(Some(_)) | Ok(None) => {
                            let mut stack = Vec::with_capacity(self.depth);
                            stack.push(self.name.into());
                            let err = ValidationError::new(format!("Missing key {key}"), stack);
                            return Ok(ValidationResult::Error(err));
                        }
                        Err(serde_err) => {
                            panic!();
                            let mut stack = Vec::with_capacity(self.depth);
                            stack.push(self.name.into());
                            let err = ValidationError::new(format!("serde: {serde_err}"), stack);
                            return Ok(ValidationResult::Error(err));
                        }
                    };
                    match map.next_value_seed(SerdeValidator { schema, depth: self.depth + 1, name: key.as_str() }) {
                        Ok(ValidationResult::Error(mut err)) => {
                            err.push_segment(self.name);
                            return Ok(ValidationResult::Error(err));
                        },
                        Ok(_) => continue,
                        Err(serde_err) => {
                            let mut stack = Vec::with_capacity(self.depth);
                            stack.push(key.clone());
                            let err = ValidationError::new("Unexpected value", stack);
                            return Ok(ValidationResult::Error(err));
                        }
                    };
                }
                Ok(ValidationResult::Success)
            }
        }
    }
}

/// Validate JSON instance against this validator.
///
/// # Errors
///
/// Returns `ValidationError` if the input instance is not valid under the given validator.
///
/// # Panics
///
/// This function panics on invalid schema.
pub fn validate(instance: &str, _schema: &Schema) -> Result<(), ValidationError> {
    Validator::new(&serde_json::Value::default()).unwrap().validate(instance)
}

#[cfg(test)]
mod tests {
    use super::{Schema, validate, Type};
    use serde_json::json;

    #[test]
    fn test_error_message() {
        let instance: serde_json::Value = serde_json::from_str(r#"{
          "inner": {
            "inner": {
              "inner": {
                "inner": {
                  "another": 1
                }
              }
            }
          }
        }"#).unwrap();
        dbg!(&instance);
        let json = serde_json::to_string_pretty(&instance).unwrap();
        println!("{json}");
        let error =
            validate(&json, /* this isn't used */ &Schema::Type(Type::Number)).expect_err("Should fail");
        assert_eq!(
            error.to_string(),
            "1 is not of type 'string' at /inner/inner/inner/inner/another"
        );
        assert_eq!(json!(instance).pointer(&error.location_pointer()), Some(&json!(1)));
    }
}
