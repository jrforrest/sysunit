use super::common::{ws, VResult, label};

use nom::{
    bytes::complete::tag,
    combinator::map,
    sequence::{preceded, delimited},

};

/// Returns the unit name parsed from a unit header line
pub fn header(input: &str) -> VResult<String> {
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
    fn test_header() {
        let input = "# [ Unit1 ]";
        let (rest, result) = header(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(result, "Unit1");

        let input = "  #  [  Blaaaarp ]  ";
        let (rest, result) = header(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(result, "Blaaaarp");

        let input = "# This is a normal comment";
        let res = header(input);
        assert!(res.is_err());
    }
}
