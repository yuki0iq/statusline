use crate::{Block, Environment, IconMode, Pretty, Style as _, file};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Hash, PartialEq, Eq)]
enum Kind {
    Cargo,
    Cmake,
    Configure,
    Makefile,
    Meson,
    Jr,
    Nix,
    Kks,
    Gradle,
    Pyproject,
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{}",
            match *self {
                Self::Cargo => "cargo",
                Self::Cmake => "cmake",
                Self::Configure => "./configure",
                Self::Makefile => "make",
                Self::Meson => "meson",
                Self::Jr => "./jr",
                Self::Nix => "nix",
                Self::Kks => "kks",
                Self::Gradle => "gradle",
                Self::Pyproject => "uv",
            }
        )
    }
}

pub struct BuildInfo(Vec<Kind>);

super::register_block!(BuildInfo);

impl Block for BuildInfo {
    fn new(environ: &Environment) -> Option<Box<dyn Block>> {
        let workdir = &environ.work_dir;
        let mut bi = vec![];

        if file::points_to_file("default.nix") {
            bi.push(Kind::Nix);
        }

        if file::points_to_file("meson.build") {
            bi.push(Kind::Meson);
        }

        if file::points_to_file("CMakeLists.txt") {
            bi.push(Kind::Cmake);
        }

        if file::points_to_file("configure") {
            bi.push(Kind::Configure);
        }

        if file::points_to_file("Makefile") {
            bi.push(Kind::Makefile);
        }

        if file::points_to_file("jr") {
            bi.push(Kind::Jr);
        }

        if file::upfind(workdir, "Cargo.toml").is_some() {
            bi.push(Kind::Cargo);
        }

        if file::upfind(workdir, "pyproject.toml").is_some() {
            bi.push(Kind::Pyproject);
        }

        if file::upfind(workdir, ".kks-workspace").is_some() {
            bi.push(Kind::Kks);
        }

        if file::points_to_file("gradle.properties") {
            bi.push(Kind::Gradle);
        }

        if bi.is_empty() {
            None
        } else {
            Some(Box::new(Self(bi)))
        }
    }
}

impl Pretty for BuildInfo {
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, _: IconMode) -> std::fmt::Result {
        let Self(buildinfo) = &self;
        let text = "[".to_owned()
            + &buildinfo
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
            + "]";
        write!(f, "{}", text.visible().purple().with_reset().invisible())
    }
}
