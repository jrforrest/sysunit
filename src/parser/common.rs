//! Common parsing functions used by the various parsers
use nom::{
    IResult,
    Parser,
    error::{ParseError, VerboseError},
    bytes::complete::{take_while1, tag},
    branch::alt,
    combinator::map,
    sequence::delimited,
    character::complete::multispace0,
};

pub type VResult<'a, O> = IResult<&'a str, O, VerboseError<&'a str>>;
use crate::models::val::ValueType;

pub fn ws<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where
    E: ParseError<&'a str>,
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn label(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    ws(take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == '.'))(input)
}

pub fn value_type(input: &str) -> IResult<&str, ValueType, VerboseError<&str>> {
    map(ws(alt((tag("string"), tag("int"), tag("bool"), tag("float")))), |value| {
        ValueType::from_str(value).expect("Invalid parsed type!")
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label() {
        let input = "one";
        let (rest, result) = label(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(result, "one");

        let input = "  one  ";
        let (rest, result) = label(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(result, "one");

        let input = "  one-two_three.four  ";
        let (rest, result) = label(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(result, "one-two_three.four");
    }

    #[test]
    fn test_arg_type() {
        let input = "  int  ";
        let (rest, result) = value_type(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(result, ValueType::Int);
    }
}
