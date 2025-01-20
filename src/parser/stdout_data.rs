//! Unit processes communicate to sysunit using messages delmited by headers and ASCII data characters
//!
//! These are streaming parsers, since the messages are sent over a pipe which is
//! read incrementally.

use super::common::{ws, VResult};

use crate::models::emit::{Header, Message};
use crate::models::stdout_data::StdoutData;

use nom::{
    bytes::streaming::{tag, take_until, take_while1},
    sequence::{preceded, tuple, terminated},
    branch::alt,
    combinator::map,
};


const START_OF_HEADER: &str = "\x01";
const START_OF_TEXT: &str = "\x02";
const END_OF_TEXT: &str = "\x03";

pub fn header_label(input: &str) -> VResult<&str> {
    ws(take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-'))(input)
}

fn text(input: &str) -> VResult<&str> {
    map(
        tuple((
            take_until(END_OF_TEXT),
            tag(END_OF_TEXT),
        )),
        |(text, _)| text
    )(input)
}

fn header(input: &str) -> VResult<Header> {
    preceded(
        tag(START_OF_HEADER),
        terminated(ws(header_text), tag(START_OF_TEXT))
    )(input)
}

fn header_text(input: &str) -> VResult<Header> {
    
    alt((
        map(
            tuple((
                header_label,
                tag("."),
                header_label,
            )),
            |(name, _, field)| Header::build(name, Some(field)),
        ),
        map(header_label, |l| Header::build(l, None)),
    ))(input)
}

fn message(input: &str) -> VResult<Message> {
    map(
        tuple((
            header,
            text,
        )),
        |(header, text)| Message {
            header,
            text: text.to_string(),
        }
    )(input)
}

fn text_line(input: &str) -> VResult<&str> {
    use nom::bytes::streaming::take_until;
    terminated(take_until("\n"), tag("\n"))(input)
}

pub fn stdout_data(input: &str) -> VResult<StdoutData> {
    alt((
        map(
            ws(message),
            StdoutData::Message
        ),
        map(text_line, |text| StdoutData::TextLine(text.to_string()))
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header() {
        let input = "\x01foo\x02";
        let (rest, result) = header(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(result, Header::build("foo", None));
    }

    #[test]
    fn test_header_text() {
        let input = "foo.bar\x02";
        let (rest, result) = header_text(input).unwrap();
        assert_eq!(rest, "\x02");
        assert_eq!(result, Header::build("foo", Some("bar")));
    }

    #[test]
    fn test_text() {
        let input = "foo\x03";
        let (rest, result) = text(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(result, "foo");
    }

    #[test]
    fn test_message() {
        let input = "\x01foo\x02bar\x03";
        let (rest, result) = message(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(result, Message {
            header: Header::build("foo", None),
            text: "bar".to_string(),
        });
    }

    #[test]
    fn test_data_line() {
        let input = "foo\n";
        let (rest, result) = text_line(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(result, "foo");
    }

    #[test]
    fn test_stdout_data() {
        let input = "foo\n";
        let (rest, result) = stdout_data(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(result, StdoutData::TextLine("foo".to_string()));

        let input = "\x01foo\x02bar\x03\n";
        let (rest, result) = stdout_data(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(result, StdoutData::Message(Message {
            header: Header::build("foo", None),
            text: "bar".to_string(),
        }));

        let input = "\n\x01meta.params\x02!message:string, !foo_count:int\x03\n\x01status\x020\x03";
        let (rest, result) = stdout_data(input).unwrap();
        assert_eq!(rest, "\x01status\x020\x03");
        assert_eq!(result, StdoutData::Message(Message {
            header: Header::build("meta", Some("params")),
            text: "!message:string, !foo_count:int".to_string(),
        }));
    }
}
