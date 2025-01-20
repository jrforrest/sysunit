//! Representation of a dependency for a unit
use super::{ValueType, ValueSet, Unit, Target};

/// When one unit depends on another, its name, args and captures must be tracked
#[derive(Debug)]
pub struct Dependency {
    pub name: String,
    pub args: ValueSet,
    pub captures: Vec<CaptureDefinition>,
    pub target: Option<Target>,
}

impl From<Dependency> for Unit {
    fn from(val: Dependency) -> Self {
        Unit::new(val.name, val.args, val.target)
    }
}

/// Emitted values which should be captured from a dependency by the
/// dependent unit
#[derive(Debug, PartialEq, Eq)]
pub struct CaptureDefinition {
    pub name: String,
    pub value_type: ValueType,
    pub required: bool,
    pub alias: Option<String>,
}

