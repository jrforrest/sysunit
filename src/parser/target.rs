use crate::models::Target;

use nom::{
    bytes::complete::tag,
    combinator::map,
    sequence::tuple,
};

use super::common::{label, VResult};

pub fn target(input: &str) -> VResult<Target> {
    map(
        tuple((
            label,
            tag("@"),
            label,
        )),
        |(user, _, host)| Target::new(user, host)
    )(input)
}
