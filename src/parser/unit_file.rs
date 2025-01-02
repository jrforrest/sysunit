use super::common::{ws, VResult, label};

use nom::{
    bytes::complete::tag,
    combinator::map,
    sequence::{preceded, delimited},
};

/// Returns the unit name parsed from a unit header line
fn unit_header(input: &str) -> VResult<String> {
    map(
        preceded(
            ws(tag("#")),
            ws(delimited(
                ws(tag("[")),
                label,
                ws(tag("]")),
            ))
        ),
        |name| name.to_string()
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_header() {
        let input = "# [ Unit1 ]";
        let (rest, result) = unit_header(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(result, "Unit1");

        let input = "  #  [  Blaaaarp ]  ";
        let (rest, result) = unit_header(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(result, "Blaaaarp");

        let input = "# This is a normal comment";
        let res = unit_header(input);
        assert!(res.is_err());
    }
}
