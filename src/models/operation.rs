//! Operation which is to be applied to a unit in an execution
use std::str::FromStr;
use std::fmt;

use anyhow::{anyhow, Result, Context};

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
    Failed(u8),
}

impl OpStatus {
    pub fn from_code(code_str: &str) -> Result<OpStatus> {
        let i = code_str.parse::<u8>().
            context(format!("Failed to parse emitted status code string: {}", code_str))?;
        match i {
            0 => Ok(OpStatus::Ok),
            n => Ok(OpStatus::Failed(n)),
        }
    }

    pub fn expect_ok(&self) -> Result<()> {
        match self {
            OpStatus::Ok => Ok(()),
            OpStatus::Failed(n) => Err(anyhow!("Operation failed on unit. Exit Code: {}", n)),
        }
    }
}

pub type CheckPresence = bool;

#[derive(Debug, Clone)]
pub enum OpCompletion {
    Check(CheckPresence),
    Apply,
    Remove,
    Deps,
    Meta,
}
