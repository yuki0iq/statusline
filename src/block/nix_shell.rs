use crate::{Block, Color, Environment, Icon, IconMode, Pretty, Style, WithStyle as _};
use std::{ffi::OsStr, path::Path};

pub struct NixShell {
    // IN_NIX_SHELL=impure
    // IN_NIX_SHELL=pure
    purity: bool,
    // buildInputs=/nix/store/HASH-derivation /nix/store/HASH-derivation ...
    inputs: Vec<String>,
}

super::register_block!(NixShell);

impl Block for NixShell {
    fn new(_: &Environment) -> Option<Self> {
        let purity = match std::env::var("IN_NIX_SHELL").ok()?.as_ref() {
            "impure" => false,
            "pure" => true,

            // Should not happen naturally
            _ => return None,
        };

        // XXX: Do we also need propagatedXxxInputs?
        let inputs = std::env::var("buildInputs")
            .unwrap_or_default()
            .split_whitespace()
            .chain(
                std::env::var("nativeBuildInputs")
                    .unwrap_or_default()
                    .split_whitespace(),
            )
            .map(Path::new)
            .filter_map(Path::file_name)
            .filter_map(OsStr::to_str)
            .filter_map(|s| s.split_once('-'))
            // This format is weird. But at least it shows a bit of hash!
            .map(|(h, p)| format!("{}:{p}", &h[..6]))
            .collect();

        Some(NixShell { purity, inputs })
    }
}

impl Pretty for NixShell {
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, mode: IconMode) -> std::fmt::Result {
        f.with_style(Color::BRIGHT_BLUE, Style::empty(), |f| {
            let purity = if self.purity { "" } else { "!" };
            write!(f, "[{purity}{}", self.icon(mode))?;
            for input in &self.inputs {
                write!(f, " {input}")?;
            }
            write!(f, "]")?;
            Ok(())
        })
    }
}

impl Icon for NixShell {
    fn icon(&self, mode: IconMode) -> &'static str {
        use IconMode::*;
        match &mode {
            Text => "nix",
            Icons | MinimalIcons => "󱄅",
        }
    }
}
