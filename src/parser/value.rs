/// Parses values used for args and emits

use crate::models::val::Value;

use super::common::{VResult, label, ws};

use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    branch::alt,
    bytes::complete::{take_until1, take_while1},
    character::complete::i32,
    sequence::{terminated, separated_pair},
    combinator::{recognize, map},
};

fn string(input: &str) -> VResult<Value> {
    let (rest, quote) = alt((tag("\""), tag("'")))(input)?;
    let (rest, value) = terminated(take_until1(quote), tag(quote))(rest)?;
    Ok((rest, Value::String(value.to_string())))
}

fn int(input: &str) -> VResult<Value> {
    map(i32, |value: i32| Value::Int(value))(input)
}

fn float(input: &str) -> VResult<Value> {
    let get_number = recognize(separated_pair(digit1, tag("."), digit1));
    map(get_number, |value: &str| {
        Value::Float(value.parse::<f32>().unwrap())
    })(input)
}

fn bool(input: &str) -> VResult<Value> {
    let (rest, val) = alt((tag("true"), tag("false")))(input)?;
    Ok((rest, Value::Bool(val == "true")))
}

fn unquoted_string(input: &str) -> VResult<Value> {
    map(take_while1(|c| c != ' ' && c != '\t' && c != '\n' && c != ','), |value: &str| {
        Value::String(value.to_string())
    })(input)
}

pub fn value(input: &str) -> VResult<Value> {
    ws(alt((string, float, int, bool, unquoted_string)))(input)
}

pub fn named_value(input: &str) -> VResult<(&str, Value)> {
    separated_pair(label, tag("="), value)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let input = "\"blarp\"";
        match string(input).unwrap() {
            (rest, Value::String(parsed_string)) => {
                assert_eq!(rest, "");
                assert_eq!(parsed_string, "blarp");
            }, _ => panic!("unexpected Value type"),
        }

        let input = "'blarp hi world !@#'";
        match string(input).unwrap() {
            (rest, Value::String(parsed_string)) => {
                assert_eq!(rest, "");
                assert_eq!(parsed_string, "blarp hi world !@#");
            },
            _ => panic!("unexpected Value type"),
        }
    }

    #[test]
    fn test_float() {
        let input = "123.456";
        match float(input).unwrap() {
            (rest, Value::Float(parsed_float)) => {
                assert_eq!(rest, "");
                assert!(parsed_float - 123.456 < 0.0001);
            },
            _ => panic!("unexpected Value type"),
        }

        let input = "123.0";
        let(rest, value) = float(input).unwrap();
        assert_eq!(rest, "");
        assert!(value.float_approx_equals(123.0))
    }

    #[test]
    fn test_bool() {
        let input = "true";
        match bool(input).unwrap() {
            (rest, Value::Bool(parsed_bool)) => {
                assert_eq!(rest, "");
                assert!(parsed_bool);
            },
            _ => panic!("unexpected Value type"),
        }

        let input = "false";
        match bool(input).unwrap() {
            (rest, Value::Bool(parsed_bool)) => {
                assert_eq!(rest, "");
                assert!(!parsed_bool);
            },
            _ => panic!("unexpected Value type"),
        }
    }

    #[test]
    fn test_int() {
        let input = "123";
        match int(input).unwrap() {
            (rest, Value::Int(parsed_int)) => {
                assert_eq!(rest, "");
                assert_eq!(parsed_int, 123);
            },
            _ => panic!("unexpected Value type"),
        }
    }

    #[test]
    fn test_unquoted_string() {
        let input = "hello ";
        match unquoted_string(input).unwrap() {
            (rest, Value::String(parsed_string)) => {
                assert_eq!(rest, " ");
                assert_eq!(parsed_string, "hello");
            },
            _ => panic!("unexpected Value type"),
        }
    }
}
