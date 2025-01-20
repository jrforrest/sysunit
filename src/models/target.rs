use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Target {
    pub proto: String,
    pub user: Option<String>,
    pub host: String,
}

impl Target {
    pub fn new(proto: &str, user: Option<&str>, host: &str) -> Self {
        Self {
            proto: proto.to_string(),
            user: user.map(|u| u.to_string()),
            host: host.to_string(),
        }
    }

    pub fn user_host_string(&self) -> String {
        match &self.user {
            Some(user) => format!("{}@{}", user, self.host),
            None => self.host.clone(),
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let user_str = match &self.user {
            Some(user) => format!("{}@", user),
            None => "".to_string(),
        };

        write!(f, "{}://{}{}", self.proto, user_str, self.host)
    }
}
