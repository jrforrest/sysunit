/*
 * Example Deps Representation:
 * pkg name=python, dir.sh path='./tmp', curl.sh:>2.3 -> binary_path:curl_binary_path:string;
 * ^-- name                ^-^-+--^---^  ^----+-----^  ^    ^             ^            ^
 *                           /  |  \          |        |  capture name  capture alias capture type
 *                          /  arg  \     tagged name arrow
 *                          |        \
 *                        arg_name arg_value
 */


use crate::models::{Dependency, version::VersionSpecification, Target};

use nom::{
    combinator::opt,
    bytes::complete::{take_while1, tag},
    multi::separated_list1,
    sequence::{tuple, preceded, terminated},
};

use super::{
    arg_values::args,
    captures::captures,
    version::version_spec,
    common::{VResult, ws},
    target::target,
};

pub fn deps(input: &str) -> VResult<Vec<Dependency>> {
    separated_list1(tag(","), dep)(input)
}

// labels in deps allow paths
fn label(input: &str) -> VResult<&str> {
    ws(take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/'))(input)
}

fn dep(input: &str) -> VResult<Dependency> {
    let (rest, target) = target_tag(input)?;
    let (rest, (name, _version)) = tagged_name(rest)?;
    let (rest, args) = args(rest)?;
    let (rest, captures) = opt(captures)(rest)?;

    Ok((
        rest,
        Dependency {
            name: name.to_string(),
            args,
            captures: captures.unwrap_or_default(),
            target,
        }
    ))
}

fn target_tag(input: &str) -> VResult<Option<Target>> {
    opt(terminated(target, tag(":")))(input)
}

fn tagged_name(input: &str) -> VResult<(&str, Option<VersionSpecification>)> {
    tuple((label, opt(preceded(tag(":"), version_spec))))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let input = "curl.sh url=\"https://placekitten.com/200/200\", output=\"/tmp/cat.png\"";
        let (rest, dep) = dep(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(dep.name, "curl.sh");
        assert_eq!(dep.args.values.len(), 2);
        assert!(dep.args.values.get("url").unwrap().string_equals("https://placekitten.com/200/200"));
        assert!(dep.args.values.get("output").unwrap().string_equals("/tmp/cat.png"));
        assert_eq!(dep.captures.len(), 0);
        assert_eq!(dep.target, None);
    }

    #[test]
    fn test_captures() {
        use crate::models::ValueType;

        let input = "curl.sh url=\"https://placekitten.com/200/200\", output=\"/tmp/cat.png\" -> size:file_size:int";
        let (rest, dep) = dep(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(dep.name, "curl.sh");
        assert_eq!(dep.args.values.len(), 2);
        assert!(dep.args.values.get("url").unwrap().string_equals("https://placekitten.com/200/200"));
        assert!(dep.args.values.get("output").unwrap().string_equals("/tmp/cat.png"));
        assert_eq!(dep.captures.len(), 1);
        assert_eq!(dep.captures[0].name, "size");
        assert_eq!(dep.captures[0].value_type, ValueType::Int);
        assert!(!dep.captures[0].required);
        assert_eq!(dep.captures[0].alias, Some("file_size".to_string()));
        assert_eq!(dep.target, None);
    }

    #[test]
    fn test_tagged_name() {
        let input = "curl.sh:>2.3";
        let (rest, (name, version_specification)) = tagged_name(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(name, "curl.sh");
        let version = version_specification.unwrap().version;
        assert_eq!(version.major, 2);
        assert_eq!(version.minor.unwrap(), 3);

        let input = "curl.sh";
        let (rest, (name, version)) = tagged_name(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(name, "curl.sh");
        assert!(version.is_none());
    }

    #[test]
    fn test_target_tag() {
        let input = "ssh://jack@localhost:";
        let (rest, target) = target_tag(input).unwrap();
        let target = target.unwrap();

        assert_eq!(rest, "");
        assert_eq!(target.user, Some("jack".into()));
        assert_eq!(&target.host, "localhost");
        assert_eq!(&target.proto, "ssh");
    }
}
