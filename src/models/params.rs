//! Parameters the specifications for arguments that can be received by units

use super::val::ValueType;

#[derive(Debug, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub value_type: ValueType,
    pub required: bool,
}
