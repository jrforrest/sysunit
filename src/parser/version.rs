//! Parses version tags with comparators, for example >=3.2.1
use super::common::VResult;
use nom::{
    character::complete::{digit1, u32},
    combinator::{opt, value},
    sequence::{tuple, preceded},
    bytes::complete::tag,
    branch::alt,
};

use crate::models::version::{Version, Comparator, VersionSpecification};

pub fn version_spec(input: &str) -> VResult<VersionSpecification> {
    let (rest, (comparator, version)) = tuple((opt(comparator), version))(input)?;
    let spec = VersionSpecification { comparator, version };

    return Ok((rest, spec))
}

pub fn version(input: &str) -> VResult<Version> {
    let (rest, (major, minor, patch)) = tuple((
        digit1,
        opt(preceded(tag("."), digit1)),
        opt(preceded(tag("."), digit1)),
    ))(input)?;

    let (_, major) = u32(major)?;
    let minor = match minor {
        Some(s) => Some(u32(s)?.1),
        None => None
    };
    let patch = match patch {
        Some(s) => Some(u32(s)?.1),
        None => None
    };

    Ok((rest, Version { major, minor, patch }))
}

fn comparator(input: &str) -> VResult<Comparator> {
    alt((
        value(Comparator::GreaterThanOrEqual, tag(">=")),
        value(Comparator::LessThanOrEqual, tag("<=")),
        value(Comparator::GreaterThan, tag(">")),
        value(Comparator::LessThan, tag("<")),
        value(Comparator::Equal, tag("="))
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparator() {
        let input = ">=";
        let (rest, result) = comparator(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(result, Comparator::GreaterThanOrEqual);
    }

    #[test]
    fn test_version_spec() {
        let input = ">=1.2.3";
        let (rest, result) = version_spec(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(result.version.major, 1);
        assert_eq!(result.version.minor, Some(2));
        assert_eq!(result.version.patch, Some(3));
        assert_eq!(result.comparator, Some(Comparator::GreaterThanOrEqual));
    }
}
