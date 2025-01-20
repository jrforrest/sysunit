//! Values are used for arguments to units and emit captures.  This module provides
//! facilities for serialization, organization and comparison of these values.

use serde::Deserialize;
use sha1::{Sha1, Digest};
use anyhow::Result;

use std::fmt;
use std::collections::HashMap;


#[derive(Debug, PartialEq, Eq, Deserialize)]
pub enum ValueType {
    String,
    Int,
    Bool,
    Float
}

/// Denotes the type of a value
impl ValueType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "string" => Ok(ValueType::String),
            "int" => Ok(ValueType::Int),
            "bool" => Ok(ValueType::Bool),
            "float" => Ok(ValueType::Float),
            _ => Err(anyhow::anyhow!("Invalid type: {}", s))
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueType::String => write!(f, "string"),
            ValueType::Int => write!(f, "int"),
            ValueType::Bool => write!(f, "bool"),
            ValueType::Float => write!(f, "float"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
}

impl Value {
    pub fn from_string(s: &str) -> Self {
        if s == "true" || s == "false" {
            return Value::Bool(true);
        }

        if let Ok(i) = s.parse::<i32>() {
            return Value::Int(i);
        }

        if let Ok(f) = s.parse::<f32>() {
            return Value::Float(f);
        }

        Value::String(s.to_string())
    }

    // Provides a display-friendly representation of the value,
    // truncated if necessary.
    pub fn tag(&self, max_len: usize) -> String {
        match self {
            Value::String(s) => {
                if s.len() > max_len {
                    format!("{}...", &s[..max_len])
                } else {
                    s.clone()
                }
            },
            Value::Int(i) => format!("{}", i),
            Value::Float(f) => format!("{}", f),
            Value::Bool(b) => format!("{}", b),
        }
    }

    #[cfg(test)]
    pub fn string_equals(&self, s: &str) -> bool {
        match self {
            Value::String(s2) => s == s2,
            _ => panic!("Cannot compare non-string value to string"),
        }
    }

    #[cfg(test)]
    pub fn int_equals(&self, i: i32) -> bool {
        match self {
            Value::Int(i2) => i == *i2,
            _ => panic!("Cannot compare non-int value to int"),
        }
    }

    #[cfg(test)]
    pub fn float_approx_equals(&self, f: f32) -> bool {
        match self {
            Value::Float(f2) => (f - *f2) < 0.0001,
            _ => panic!("Cannot compare non-float value to float"),
        }
    }

    #[cfg(test)]
    pub fn bool_equals(&self, b: bool) -> bool {
        match self {
            Value::Bool(b2) => b == *b2,
            _ => panic!("Cannot compare non-bool value to bool"),
        }
    }

    pub fn get_type(&self) -> ValueType {
        match self {
            Value::String(_) => ValueType::String,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::Bool(_) => ValueType::Bool,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}


/// A value set is a key-value collection of values.  It is used for
/// primarily for emit captures and unit args.
#[derive(Debug, Clone)]
pub struct ValueSet {
    pub values: HashMap<String, Value>,
}

impl Default for ValueSet {
    fn default() -> Self {
        Self::new()
    }
}

impl ValueSet {
    pub fn new() -> Self {
        ValueSet {
            values: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn merge(&mut self, other: &ValueSet) {
        for (k, v) in &other.values {
            self.values.insert(k.clone(), v.clone());
        }
    }

    /* Provides SHA1sum of all values combined */
    pub fn get_sig(&self) -> String {
        let mut all = String::new();
        for (k, v) in &self.values {
            all.push_str(format!("{}={}", k, v).as_str());
        }
        format!("{:x}", Sha1::digest(all.as_bytes()))
    }

    pub fn tag(&self) -> String {
        let mut tag = String::new();
        for (k, v) in &self.values {
            tag.push_str(format!("{}={}", k, v.tag(10)).as_str());
        }
        tag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_set() {
        let mut value_set = ValueSet::new();
        value_set.add_value("foo", Value::String("bar".to_string()));
        value_set.add_value("bar", Value::Int(123));
        value_set.add_value("blarp", Value::Float(432.34));
        value_set.add_value("blip", Value::Bool(true));

        assert_eq!(value_set.values.len(), 4);
        assert!(value_set.get("foo").unwrap().string_equals("bar"));
        assert!(value_set.get("bar").unwrap().int_equals(123));
        assert!(value_set.get("blarp").unwrap().float_approx_equals(432.34));
        assert!(value_set.get("blip").unwrap().bool_equals(true));
    }
}
