use crate::models::Target;

use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::tuple,
};

use super::common::{label, VResult};

pub fn target(input: &str) -> VResult<Target> {
    map(
        tuple((
            proto,
            opt(user),
            label,
        )),
        |(proto, user, host)| Target::new(proto, user, host)
    )(input)
}

fn user(input: &str) -> VResult<&str> {
    map(
        tuple((
            label,
            tag("@"),
        )),
        |(user, _)| user
    )(input)
}

fn proto(input: &str) -> VResult<&str> {
    map(
        tuple((
            label,
            tag("://"),
        )),
        |(proto, _)| proto
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target() {
        let input = "ssh://user@host";
        let result = target(input);
        assert_eq!(result, Ok(("", Target::new("ssh", Some("user"), "host"))));

        let input = "ssh://host";
        let result = target(input);
        assert_eq!(result, Ok(("", Target::new("ssh", None, "host"))));
    }
}
