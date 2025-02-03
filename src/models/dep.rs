//! Representation of a dependency for a unit
use anyhow::{Result, anyhow};
use super::{ValueType, ValueSet, Target};

#[derive(Debug, Clone)]
pub struct Dependencies {
    pub units: Vec<Dependency>,
    pub files: Vec<FileDependency>,
}

impl Dependencies {
    pub fn new() -> Dependencies {
        Dependencies {
            units: Vec::new(),
            files: Vec::new(),
        }
    }
}

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

/// File that a unit depends on
#[derive(Debug, Clone)]
pub struct FileDependency {
    pub src: String,
    pub dest: String,
}

impl FileDependency {
    pub fn from_args(args: ValueSet) -> Result<FileDependency> {
        let src = args.get("src").ok_or(anyhow!("File dependency missing src argument"))?.to_string();
        let dest = args.get("dest").ok_or(anyhow!("File dependency missing dest argument"))?.to_string();
        Ok(FileDependency { src, dest })
    }
}
