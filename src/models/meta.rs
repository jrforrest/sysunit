//! Contains meta emitted from a unit
use super::Param;

#[derive(Debug)]
pub struct Meta {
    pub author: Option<String>,
    pub desc: Option<String>,
    pub version: Option<String>,
    pub params: Vec<Param>,
}

impl Meta {
    pub fn empty() -> Meta {
        Meta {
            author: None,
            desc: None,
            version: None,
            params: Vec::new(),
        }
    }
}
