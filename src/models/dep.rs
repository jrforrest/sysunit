//! Representation of a dependency for a unit
use super::{ValueType, ValueSet, Target};

/// When one unit depends on another, its name, args and captures must be tracked
#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub args: ValueSet,
    pub captures: Vec<CaptureDefinition>,
    pub target: Option<Target>,
}

/// Emitted values which should be captured from a dependency by the
/// dependent unit
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CaptureDefinition {
    pub name: String,
    pub value_type: ValueType,
    pub required: bool,
    pub alias: Option<String>,
}
