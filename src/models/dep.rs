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

impl Into<Unit> for Dependency {
    fn into(self) -> Unit {
        Unit::new(self.name, self.args, self.target)
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

