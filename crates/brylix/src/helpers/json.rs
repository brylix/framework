//! JSON column helpers for working with `serde_json::Value` fields.
//!
//! When using SeaORM with JSON columns, values are often stored as
//! `Option<serde_json::Value>`. This module provides extension traits
//! for ergonomic parsing of these values.
//!
//! # Usage
//!
//! ```rust
//! use brylix::helpers::json::JsonValueExt;
//! use serde_json::json;
//!
//! let value = Some(json!({"name": "John", "age": 30}));
//!
//! // Parse into a typed struct
//! let name: Option<String> = value.parse_as();
//!
//! // Parse with a default fallback
//! let tags: Vec<String> = None::<serde_json::Value>.parse_or_default();
//! assert!(tags.is_empty());
//! ```

use serde::de::DeserializeOwned;
use serde_json::Value;

/// Extension trait for parsing JSON values into typed Rust structures.
pub trait JsonValueExt {
    /// Parse the JSON value into the target type.
    ///
    /// Returns `None` if the value is `None` or deserialization fails.
    fn parse_as<T: DeserializeOwned>(&self) -> Option<T>;

    /// Parse the JSON value into the target type, falling back to `Default`.
    ///
    /// Returns `T::default()` if the value is `None` or deserialization fails.
    fn parse_or_default<T: DeserializeOwned + Default>(&self) -> T;
}

impl JsonValueExt for Option<Value> {
    fn parse_as<T: DeserializeOwned>(&self) -> Option<T> {
        self.as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    fn parse_or_default<T: DeserializeOwned + Default>(&self) -> T {
        self.parse_as().unwrap_or_default()
    }
}

impl JsonValueExt for Value {
    fn parse_as<T: DeserializeOwned>(&self) -> Option<T> {
        serde_json::from_value(self.clone()).ok()
    }

    fn parse_or_default<T: DeserializeOwned + Default>(&self) -> T {
        self.parse_as().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        age: u32,
    }

    #[test]
    fn test_parse_as_option_value_some() {
        let value: Option<Value> = Some(json!({"name": "John", "age": 30}));
        let result: Option<TestData> = value.parse_as();
        assert_eq!(
            result,
            Some(TestData {
                name: "John".to_string(),
                age: 30
            })
        );
    }

    #[test]
    fn test_parse_as_option_value_none() {
        let value: Option<Value> = None;
        let result: Option<TestData> = value.parse_as();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_as_option_value_invalid() {
        let value: Option<Value> = Some(json!("not a struct"));
        let result: Option<TestData> = value.parse_as();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_or_default_option_value() {
        let value: Option<Value> = Some(json!(["a", "b", "c"]));
        let result: Vec<String> = value.parse_or_default();
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_parse_or_default_option_value_none() {
        let value: Option<Value> = None;
        let result: Vec<String> = value.parse_or_default();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_as_value() {
        let value = json!({"name": "Jane", "age": 25});
        let result: Option<TestData> = value.parse_as();
        assert_eq!(
            result,
            Some(TestData {
                name: "Jane".to_string(),
                age: 25
            })
        );
    }

    #[test]
    fn test_parse_or_default_value() {
        let value = json!([1, 2, 3]);
        let result: Vec<i32> = value.parse_or_default();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_or_default_value_invalid() {
        let value = json!("not a vec");
        let result: Vec<i32> = value.parse_or_default();
        assert!(result.is_empty());
    }
}
