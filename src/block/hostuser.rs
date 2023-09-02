use crate::{Chassis, Environment, Icon, IconMode, Pretty, SimpleBlock, Style};

struct Host(Chassis, String);
struct User(String);
pub struct HostUser(User, Host);

impl SimpleBlock for HostUser {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl From<&Environment> for Host {
    fn from(env: &Environment) -> Self {
        Host(env.chassis, env.host.clone())
    }
}

impl From<&Environment> for User {
    fn from(env: &Environment) -> Self {
        User(env.user.clone())
    }
}

impl From<&Environment> for HostUser {
    fn from(env: &Environment) -> Self {
        HostUser(User::from(env), Host::from(env))
    }
}

impl Icon for Host {
    fn icon(&self, mode: &IconMode) -> &'static str {
        self.0.icon(mode)
    }
}

impl Icon for User {
    fn icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match mode {
            Text => "as",
            Icons | MinimalIcons => "",
        }
    }
}

impl Host {
    fn pre_icon(&self, mode: &IconMode) -> &'static str {
        use IconMode::*;
        match mode {
            Text => " at ",
            Icons | MinimalIcons => "＠",
        }
    }
}

impl Pretty for Host {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        Some(
            format!("{}{} {}]", self.pre_icon(mode), self.1, self.icon(mode),)
                .colorize_with(&self.1)
                .to_string(),
        )
    }
}

impl Pretty for User {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        Some(
            format!("[{} {}", self.icon(mode), self.0)
                .colorize_with(&self.0)
                .to_string(),
        )
    }
}

impl Pretty for HostUser {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        Some(
            format!("{}{}", self.0.pretty(mode)?, self.1.pretty(mode)?)
                .bold()
                .with_reset()
                .to_string(),
        )
    }
}
