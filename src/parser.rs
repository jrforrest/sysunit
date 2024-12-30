//! Sysu has a couple of formats for things like dep strings and captures that
//! need to be parsed.

mod common;
mod deps;
mod version;
mod params;
mod arg_values;
mod captures;
mod value;
mod target;

pub mod stdout_data;

use nom::error::convert_error;
use nom::combinator::all_consuming;

use tracing::{instrument, event, Level};

use self::{
    params::params,
    deps::deps,
    version::{version, version_spec},
    value::value,
    target::target,
};

use crate::models::{Param, Dependency, Value, Target, StdoutData};
use crate::models::version::{Version, VersionSpecification};

use anyhow::{Result, anyhow};
use common::ws;

pub enum StreamingResult<'a, T> {
    Incomplete,
    Complete(Result<(&'a str, T)>),
}

#[instrument]
pub fn parse_stdout_data(input: &str) -> StreamingResult<StdoutData> {
    use StreamingResult::*;

    event!(Level::DEBUG, "Parsing stdout data: '{}'", input);
    match stdout_data::stdout_data(input) {
        Ok((remaining, data)) => {
            event!(Level::DEBUG, "Parsed stdout data: {:?}", data);
            Complete(Ok((remaining, data)))
        },
        Err(e) => {
            match e {
                nom::Err::Error(inner_e) | nom::Err::Failure(inner_e) => {
                    let fancy_error = convert_error(input, inner_e);
                    Complete(Err(anyhow!("Failed to parse emit message: {}", fancy_error.to_string())))
                },
                nom::Err::Incomplete(_) => Incomplete,
            }
        }
    }
}

/// Receives a parser which it will run with the given input, and returns the parsed result.
/// Errors are unwrapped into a string with some additional context on where parsing errors
/// occured for the user.
fn parse_with_better_errors<'a, T>(input: &'a str, parser: impl Fn(&'a str) -> common::VResult<'a, T>) -> Result<T> {
    match all_consuming(ws(parser))(input) {
        Ok((_, parsed)) => Ok(parsed),
        Err(e) => {
            match e {
                nom::Err::Error(inner_e) | nom::Err::Failure(inner_e) => {
                    let fancy_error = convert_error(input, inner_e);
                    Err(anyhow!("Failed to parse: {}", fancy_error.to_string()))
                },
                _ => {
                    Err(anyhow!("Failed to parse: {}", e.to_string()))
                },
            }
        }
    }
}


pub fn parse_params(input: &str) -> Result<Vec<Param>> {
    parse_with_better_errors(input, params)
}

pub fn parse_target(input: &str) -> Result<Target> {
    parse_with_better_errors(input, target)
}


pub fn parse_value(input: &str) -> Result<Value> {
    parse_with_better_errors(input, value)
}

pub fn parse_deps(input: &str) -> Result<Vec<Dependency>> {
    parse_with_better_errors(input, deps)
}
