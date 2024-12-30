//! Parses captures for dependencies

use crate::models::CaptureDefinition;
use super::common::{label, value_type, ws, VResult};

use nom::{
    bytes::complete::tag,
    multi::separated_list1,
    sequence::preceded,
    sequence::tuple,
    branch::alt,
    combinator::map,
};

pub fn captures(input: &str) -> VResult<Vec<CaptureDefinition>> {
    preceded(tag("->"), separated_list1(tag(","), ws(capture)))(input)
}

fn capture(input: &str) -> VResult<CaptureDefinition> {
    alt((aliased_capture, unaliased_capture))(input)
}

fn unaliased_capture(input: &str) -> VResult<CaptureDefinition> {
    let parser = tuple((
        label,
        preceded(tag(":"), value_type)
    ));

    map(parser, |(name, value_type)| CaptureDefinition {
        name: name.to_string(),
        value_type,
        required: false,
        alias: None,
    })(input)
}

fn aliased_capture(input: &str) -> VResult<CaptureDefinition> {
    let parser = tuple((
        label,
        preceded(tag(":"), label),
        preceded(tag(":"), value_type)
    ));

    map(parser, |(name, alias, value_type)| CaptureDefinition {
        name: name.to_string(),
        value_type,
        required: false,
        alias: Some(alias.to_string()),
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::val::ValueType;

    #[test]
    fn test_capture() {
        let input = "foo:string";
        let (rest, result) = capture(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(result, CaptureDefinition {
            name: "foo".to_string(),
            value_type: ValueType::String,
            required: false,
            alias: None,
        });

        let input = "foo:bar:string";
        let (rest, result) = capture(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(result, CaptureDefinition {
            name: "foo".to_string(),
            value_type: ValueType::String,
            required: false,
            alias: Some("bar".to_string()),
        });
    }
}
