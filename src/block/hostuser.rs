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

impl Pretty for Host {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        Some(
            format!("[{} {}]", self.icon(mode), self.1)
                .visible()
                .colorize_with(&self.1)
                .invisible()
                .to_string(),
        )
    }
}

impl Pretty for User {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        Some(
            format!("[{} {}]", self.icon(mode), self.0)
                .visible()
                .colorize_with(&self.0)
                .invisible()
                .to_string(),
        )
    }
}

impl Pretty for HostUser {
    fn pretty(&self, mode: &IconMode) -> Option<String> {
        Some(
            format!("{} {}", self.1.pretty(mode)?, self.0.pretty(mode)?)
                .visible()
                .bold()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}
