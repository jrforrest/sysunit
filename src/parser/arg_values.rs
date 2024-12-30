//! Parses argument values for dependencies and CLI
use crate::models::val::ValueSet;

use super::value::named_value;
use super::common::VResult;

use nom::{
    bytes::complete::tag,
    combinator::map,
    multi::separated_list0,
};

pub fn args(input: &str) -> VResult<ValueSet> {
    map(separated_list0(tag(","), named_value), |args| {
        let mut args_set = ValueSet::new();

        for (key, value) in args {
            args_set.add_value(key, value);
        }

        args_set
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args() {
        let input = "foo =\"bar\", bar=123, blarp=432.34, blip=true";
        let (rest, arg_set) = args(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(arg_set.values.len(), 4);
        assert!(arg_set.get("foo").unwrap().string_equals("bar"));
        assert!(arg_set.get("bar").unwrap().int_equals(123));
        assert!(arg_set.get("blarp").unwrap().float_approx_equals(432.34));
        assert!(arg_set.get("blip").unwrap().bool_equals(true));
    }
}
