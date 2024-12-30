//! Represents a unit of change to the system
//!
//! Units are identified uniquely by their name, arguments and target

use super::val::ValueSet;
use super::target::Target;
use std::hash::{Hash, Hasher};
use std::fmt;

use std::sync::Arc;

pub type UnitArc = Arc<Unit>;

/// A pointer to a unit.  These are disambiguated by ID.  Many unit structs with the
/// same ID may be instantiated at once.  Actual unit state is stored in the executor.
#[derive(Debug)]
pub struct Unit {
    pub name: String,
    /// The arguments provided for the unit's invocation
    pub args: ValueSet,
    pub target: Option<Target>,
}

impl Unit {
    pub fn new(name: String, args: ValueSet, target: Option<Target> ) -> Unit {
        Unit { name, args, target }
    }

    pub fn get_id(&self) -> String {
        format!("{}-{}", self.name, self.args.get_sig())
    }

    pub fn tag(&self) -> String {
        // render in the form of unit-name(arg1=val1, arg2=val2), limit content in (...) to 40
        // chars by truncating the values
        format!("{}({})", self.name, self.args.tag())
    }
}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl Eq for Unit {}

impl Hash for Unit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_id().hash(state);
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.name, self.args.get_sig())
    }
}

impl ResolvableNode for UnitArc {
    fn get_id(&self) -> String {
        Unit::get_id(&self)
    }
}

use crate::engine::ResolvableNode;
