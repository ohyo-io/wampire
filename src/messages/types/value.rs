use std::{collections::HashMap, fmt};

use itertools::Itertools;

use crate::CallResult;

use super::{CallError, Reason};

/// Alias for HashMap of [Value]'s
pub type Dict = HashMap<String, Value>;

/// Alias for Vec of [Value]'s
pub type List = Vec<Value>;

// TODO properly implement Hash and Eq

/// Represents URI
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct URI {
    /// URI string
    pub uri: String,
}

impl URI {
    /// Create new URI with param
    pub fn new(uri: &str) -> URI {
        URI {
            uri: uri.to_string(),
        }
    }
}

/// Represents Wamp Value
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    // The ID and URI types cannot be distinguished from string and integer types respectively.
    // So, we just ignore them here
    /// Dictionary value
    Dict(Dict),
    /// Integer value
    Integer(i64),
    /// Unsigned integer value
    UnsignedInteger(u64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// List value
    List(List),
    /// Boolean value
    Boolean(bool),
}

struct URIVisitor;
struct ValueVisitor;

/// Defines Argument List functonality
pub trait ArgList {
    /// Retrieve value as i64 by index
    fn get_int(&self, index: usize) -> CallResult<Option<i64>>;
    /// Retrieve value as string by index
    fn get_string(&self, index: usize) -> CallResult<Option<&str>>;
    /// Verify argument list length
    fn verify_len(&self, expected_len: usize) -> CallResult<()>;
}

/// Defines Argument Dictionary functonality
pub trait ArgDict {
    /// Retrieve value as i64 by key
    fn get_int(&self, key: &str) -> CallResult<Option<i64>>;
    /// Retrieve value as i64 by key
    fn get_string<'a>(&'a self, key: &str) -> CallResult<Option<&'a str>>;
}

impl ArgList for List {
    fn get_int(&self, index: usize) -> CallResult<Option<i64>> {
        let value = self.get(index);
        match value {
            Some(value) => {
                if let Value::Integer(value) = *value {
                    Ok(Some(value))
                } else {
                    Err(CallError::new(
                        Reason::InvalidArgument,
                        Some(vec![Value::String(format!(
                            "Expected integer, got {}",
                            value.summarize()
                        ))]),
                        None,
                    ))
                }
            }
            None => Ok(None),
        }
    }

    fn get_string(&self, index: usize) -> CallResult<Option<&str>> {
        let value = self.get(index);
        match value {
            Some(value) => {
                if let Value::String(ref value) = *value {
                    Ok(Some(value))
                } else {
                    Err(CallError::new(
                        Reason::InvalidArgument,
                        Some(vec![Value::String(format!(
                            "Expected string, got {}",
                            value.summarize()
                        ))]),
                        None,
                    ))
                }
            }
            None => Ok(None),
        }
    }

    fn verify_len(&self, expected_len: usize) -> CallResult<()> {
        if self.len() >= expected_len {
            Ok(())
        } else {
            Err(CallError::new(
                Reason::InvalidArgument,
                Some(vec![Value::String(format!(
                    "Expected {} arguments, got {}",
                    expected_len,
                    self.len()
                ))]),
                None,
            ))
        }
    }
}

impl ArgDict for Dict {
    fn get_int(&self, key: &str) -> CallResult<Option<i64>> {
        let value = self.get(key);
        match value {
            Some(value) => {
                if let Value::Integer(value) = *value {
                    Ok(Some(value))
                } else {
                    Err(CallError::new(
                        Reason::InvalidArgument,
                        Some(vec![Value::String(format!(
                            "Expected integer, got {}",
                            value.summarize()
                        ))]),
                        None,
                    ))
                }
            }
            None => Ok(None),
        }
    }
    fn get_string<'a>(&'a self, key: &str) -> CallResult<Option<&'a str>> {
        let value = self.get(key);
        match value {
            Some(value) => {
                if let Value::String(ref value) = *value {
                    Ok(Some(value))
                } else {
                    Err(CallError::new(
                        Reason::InvalidArgument,
                        Some(vec![Value::String(format!(
                            "Expected string, got {}",
                            value.summarize()
                        ))]),
                        None,
                    ))
                }
            }
            None => Ok(None),
        }
    }
}

impl Value {
    /// Represent Value as string
    pub fn summarize(&self) -> String {
        match *self {
            Value::Dict(ref d) => {
                let mut result = String::new();
                result.push('{');
                result.push_str(
                    &d.iter()
                        .take(50)
                        .map(|(key, value)| format!("{}:{}", key, value.summarize()))
                        .join(","),
                );
                result.push('}');
                result
            }
            Value::Integer(i) => i.to_string(),
            Value::UnsignedInteger(u) => u.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(ref s) => {
                if s.len() > 50 {
                    s[..50].to_string()
                } else {
                    s.clone()
                }
            }
            Value::List(ref l) => {
                let mut result = String::new();
                result.push('[');
                result.push_str(
                    &l.iter()
                        .take(50)
                        .map(|element| element.summarize())
                        .join(","),
                );
                result.push(']');
                result
            }
            Value::Boolean(b) => b.to_string(),
        }
    }
}

// XXX Right now there is no way to tell the difference between a URI and a string, or an ID and an Integer
impl<'de> serde::de::Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON value")
    }

    #[inline]
    fn visit_str<E>(self, value: &str) -> Result<Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(value.to_string()))
    }

    #[inline]
    fn visit_i64<E>(self, value: i64) -> Result<Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(value))
    }

    #[inline]
    fn visit_u64<E>(self, value: u64) -> Result<Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::UnsignedInteger(value))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Float(value))
    }

    #[inline]
    fn visit_bool<E>(self, value: bool) -> Result<Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Boolean(value))
    }

    #[inline]
    fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<Value, Visitor::Error>
    where
        Visitor: serde::de::MapAccess<'de>,
    {
        let mut values = HashMap::new();
        if let Some(size) = visitor.size_hint() {
            values.reserve(size);
        }

        while let Some((key, value)) = visitor.next_entry()? {
            values.insert(key, value);
        }

        Ok(Value::Dict(values))
    }

    #[inline]
    fn visit_seq<Visitor>(self, mut visitor: Visitor) -> Result<Value, Visitor::Error>
    where
        Visitor: serde::de::SeqAccess<'de>,
    {
        let mut values = Vec::new();
        if let Some(size) = visitor.size_hint() {
            values.reserve(size);
        }

        while let Some(value) = visitor.next_element()? {
            values.push(value);
        }

        Ok(Value::List(values))
    }
}

// Value

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match *self {
            Value::Dict(ref dict) => dict.serialize(serializer),
            Value::String(ref s) => serializer.serialize_str(s),
            Value::Integer(i) => serializer.serialize_i64(i),
            Value::UnsignedInteger(u) => serializer.serialize_u64(u),
            Value::Float(f) => serializer.serialize_f64(f),
            Value::List(ref list) => list.serialize(serializer),
            Value::Boolean(b) => serializer.serialize_bool(b),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

// URI

impl serde::Serialize for URI {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.uri)
    }
}

impl<'de> serde::Deserialize<'de> for URI {
    fn deserialize<D>(deserializer: D) -> Result<URI, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(URIVisitor)
    }
}

impl<'de> serde::de::Visitor<'de> for URIVisitor {
    type Value = URI;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("URI")
    }
    #[inline]
    fn visit_str<E>(self, value: &str) -> Result<URI, E>
    where
        E: serde::de::Error,
    {
        Ok(URI {
            uri: value.to_string(),
        })
    }
}
