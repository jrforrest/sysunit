//! Parser for paramument definitions

use nom::{
    character::complete::char,
    bytes::complete::tag,
    multi::separated_list0,
    combinator::opt,
    sequence::{preceded, tuple},
    error::context,
};

use std::str;
use crate::models::Param;
use super::common::{ws, label, value_type, VResult};

/* param format:  !foo:string, bar:int, baz:bool
 *              ^-- required ^-+---^   |   ^-- type
 *                             |       +-- name
 *                           param 
 *
 * name and type are both kinds of labels
 * */

pub fn params(input: &str) -> VResult<Vec<Param>> {
    context(
        "params",
        separated_list0(char(','), param)
    )(input)
}

fn param(input: &str) -> VResult<Param> {
    let (rest, (bang, name, value_type)) = tuple((
        opt(ws(tag("!"))),
        label,
        preceded(tag(":"), value_type)
    ))(input)?;

    let param = Param {
        name: name.to_string(),
        value_type,
        required: bang.is_some()
    };

    Ok((rest, param))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::val::ValueType;

    #[test]
    fn test_param() {
        let input = "  !foo: string  ";
        let (rest, result) = param(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(result, Param {
            name: "foo".to_string(),
            value_type: ValueType::String,
            required: true
        });
    }

    #[test]
    fn test_params() {
        let input = "  !foo: string, bar: int, baz: bool  ";
        let (rest, result) = params(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(result, vec![
            Param {
                name: "foo".to_string(),
                value_type: ValueType::String,
                required: true
            },
            Param {
                name: "bar".to_string(),
                value_type: ValueType::Int,
                required: false
            },
            Param {
                name: "baz".to_string(),
                value_type: ValueType::Bool,
                required: false
            }
        ]);
    }
}
