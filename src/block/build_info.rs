use crate::{Environment, Extend, IconMode, Pretty, Style as _, file};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Hash, PartialEq, Eq)]
enum Kind {
    Cargo,
    Cmake,
    Configure,
    Flake,
    Makefile,
    Install,
    Jr,
    NixShell,
    Qbs,
    Qmake,
    Kks,
    Gradle,
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", match *self {
            Self::Cargo => "cargo",
            Self::Cmake => "cmake",
            Self::Configure => "./configure",
            Self::Flake => "flake",
            Self::Makefile => "make",
            Self::Install => "./install",
            Self::Jr => "./jr",
            Self::NixShell => "nix-shell",
            Self::Qbs => "qbs",
            Self::Qmake => "qmake",
            Self::Kks => "kks",
            Self::Gradle => "gradle",
        })
    }
}

pub struct BuildInfo(Vec<Kind>);

impl Extend for BuildInfo {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl From<&Environment> for BuildInfo {
    fn from(env: &Environment) -> Self {
        let workdir = &env.work_dir;
        let mut bi = vec![];

        if file::points_to_file("flake.nix") {
            bi.push(Kind::Flake);
        }

        if file::points_to_file("shell.nix") {
            bi.push(Kind::NixShell);
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

        if file::points_to_file("install") {
            bi.push(Kind::Install);
        }

        if file::points_to_file("jr") {
            bi.push(Kind::Jr);
        }

        if let Ok(true) = file::exists_that(workdir, |filename| filename.ends_with(".qbs")) {
            bi.push(Kind::Qbs);
        }

        if let Ok(true) = file::exists_that(workdir, |filename| filename.ends_with(".pro")) {
            bi.push(Kind::Qmake);
        }

        if file::upfind(workdir, "Cargo.toml").is_ok() {
            bi.push(Kind::Cargo);
        }

        if file::upfind(workdir, ".kks-workspace").is_ok() {
            bi.push(Kind::Kks);
        }

        if file::points_to_file("gradle.properties") {
            bi.push(Kind::Gradle);
        }

        Self(bi)
    }
}

impl Pretty for BuildInfo {
    fn pretty(&self, _: &IconMode) -> Option<String> {
        let Self(buildinfo) = &self;
        if buildinfo.is_empty() {
            None?;
        }
        Some(
            buildinfo
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
                .boxed()
                .visible()
                .purple()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}
