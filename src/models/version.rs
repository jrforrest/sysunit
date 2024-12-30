#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Comparator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Version {
    pub major: u32,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct VersionSpecification {
    pub comparator: Option<Comparator>,
    pub version: Version
}
