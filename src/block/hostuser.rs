use crate::{Chassis, Environment, Extend, Icon, IconMode, Pretty, Style as _};

struct Host(Chassis, String);
struct User(String);
pub struct HostUser(User, Host);

impl Extend for HostUser {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl Host {
    fn new(env: &Environment) -> Self {
        Host(env.chassis, env.host.clone())
    }
}

impl User {
    fn new(env: &Environment) -> Self {
        User(env.user.clone())
    }
}

impl HostUser {
    pub fn new(env: &Environment) -> Box<dyn Extend> {
        Box::new(HostUser(User::new(env), Host::new(env)))
    }
}

impl Icon for Host {
    fn icon(&self, mode: IconMode) -> &'static str {
        self.0.icon(mode)
    }
}

impl Icon for User {
    fn icon(&self, mode: IconMode) -> &'static str {
        use IconMode::*;
        match mode {
            Text => "as",
            Icons | MinimalIcons => "",
        }
    }
}

impl Pretty for Host {
    fn pretty(&self, mode: IconMode) -> Option<String> {
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
    fn pretty(&self, mode: IconMode) -> Option<String> {
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
    fn pretty(&self, mode: IconMode) -> Option<String> {
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
