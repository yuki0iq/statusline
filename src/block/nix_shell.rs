use crate::{Environment, Block, Icon, IconMode, Pretty, Style as _};
use std::{ffi::OsStr, path::Path};

pub struct NixShell {
    // IN_NIX_SHELL=impure
    // IN_NIX_SHELL=pure
    purity: bool,
    // buildInputs=/nix/store/HASH-derivation /nix/store/HASH-derivation ...
    inputs: Vec<String>,
}

pub type MaybeNixShell = Option<NixShell>;

impl Block for MaybeNixShell {}

impl From<&Environment> for MaybeNixShell {
    fn from(_: &Environment) -> Self {
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

impl Pretty for MaybeNixShell {
    fn pretty(&self, mode: IconMode) -> Option<String> {
        // I am not proud of the number of allocations here
        self.as_ref().map(|ns| {
            format!(
                "[{}{} {}]",
                if ns.purity { "" } else { "!" },
                ns.icon(mode),
                ns.inputs.join(" ")
            )
            .visible()
            .bright_blue()
            .with_reset()
            .invisible()
            .to_string()
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
