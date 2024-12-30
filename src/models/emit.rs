//! Emit protocol data structures

use std::fmt;

/// The protocol is made up of many sections delimited by file separators
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Message {
    pub header: Header,
    pub text: String,
}

/// Headers describe the data in the section.  The have both a name, and
/// an optional field which can be used to further describe the data.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Header {
    pub name: String,
    pub field: Option<String>,
}

impl Header {
    pub fn build(name: &str, field: Option<&str>) -> Self {
        let field = field.map(|s| s.to_string());
        Self { name: name.to_string(), field}
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.field {
            Some(field) => write!(f, "{}.{}", self.name, field),
            None => write!(f, "{}", self.name),
        }
    }
}
