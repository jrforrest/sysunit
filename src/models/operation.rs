//! Operation which is to be applied to a unit in an execution
use std::str::FromStr;
use std::fmt;
use std::sync::Arc;

use anyhow::{anyhow, Result, Context};
use super::{ValueSet, Dependency, Meta};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    Check,
    Apply,
    Remove,
    Deps,
    Meta
}

impl FromStr for Operation {
    type Err = ();

    fn from_str(input: &str) -> Result<Operation, Self::Err> {
        match input {
            "check" => Ok(Self::Check),
            "apply" => Ok(Self::Apply),
            "remove" => Ok(Self::Remove),
            _ => Err(())
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Check => write!(f, "check"),
            Self::Apply => write!(f, "apply"),
            Self::Remove => write!(f, "remove"),
            Self::Deps => write!(f, "deps"),
            Self::Meta => write!(f, "meta"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OpStatus {
    Ok,
    Failed,
}

impl OpStatus {
    pub fn from_code(code_str: &str) -> Result<OpStatus> {
        let i = code_str.parse::<u8>().
            context(format!("Failed to parse emitted status code string: {}", code_str))?;
        match i {
            0 => Ok(OpStatus::Ok),
            _ => Ok(OpStatus::Failed),
        }
    }

    pub fn expect_ok(&self) -> Result<()> {
        match self {
            OpStatus::Ok => Ok(()),
            _ => Err(anyhow!("Operation failed on unit.  Status: {:?}", self)),
        }
    }
}

pub type CheckPresence = bool;

#[derive(Debug, Clone)]
pub enum OpCompletion {
    Check(OpStatus, CheckPresence, Arc<ValueSet>),
    Apply(OpStatus, Arc<ValueSet>),
    Remove(OpStatus, Arc<ValueSet>),
    Deps(OpStatus, Arc<Vec<Dependency>>),
    Meta(OpStatus, Arc<Meta>),
}

impl OpCompletion {
    pub fn expect_ok(&self) -> Result<()> {
        match self {
            OpCompletion::Check(status, _, _) => status.expect_ok(),
            OpCompletion::Apply(status, _) => status.expect_ok(),
            OpCompletion::Remove(status, _) => status.expect_ok(),
            OpCompletion::Deps(status, _) => status.expect_ok(),
            OpCompletion::Meta(status, _) => status.expect_ok(),
        }
    }
}

