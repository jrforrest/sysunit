use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Target {
    pub user: String,
    pub host: String,
}

impl Target {
    pub fn new(user: &str, host: &str) -> Self {
        Self {
            user: user.to_string(),
            host: host.to_string(),
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.user, self.host)
    }
}
