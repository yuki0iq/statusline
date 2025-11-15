use crate::{Block, Chassis, Color, Environment, Icon, IconMode, Pretty, Style, WithStyle as _};

struct Host(Chassis, String);
struct User(String);
pub struct HostUser(User, Host);

super::register_block!(HostUser);

impl Host {
    fn new(env: &Environment) -> Self {
        Host(Chassis::get(), env.host.clone())
    }
}

impl User {
    fn new(env: &Environment) -> Self {
        User(env.user.clone())
    }
}

impl Block for HostUser {
    fn new(environ: &Environment) -> Option<Box<dyn Block>> {
        Some(Box::new(HostUser(User::new(environ), Host::new(environ))))
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
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, mode: IconMode) -> std::fmt::Result {
        f.with_style(Color::of(&self.1), Style::BOLD, |f| {
            write!(f, "[{} {}]", self.icon(mode), self.1)
        })
    }
}

impl Pretty for User {
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, mode: IconMode) -> std::fmt::Result {
        f.with_style(Color::of(&self.0), Style::BOLD, |f| {
            write!(f, "[{} {}]", self.icon(mode), self.0)
        })
    }
}

impl Pretty for HostUser {
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, mode: IconMode) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            crate::icon::display(&self.1, mode),
            crate::icon::display(&self.0, mode)
        )
    }
}
