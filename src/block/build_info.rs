use crate::{file, Environment, IconMode, Pretty, SimpleBlock, Style};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Hash, PartialEq, Eq)]
pub enum BuildInfoKind {
    Cargo,
    Cmake,
    Configure,
    Makefile,
    Install,
    Jr,
    Qbs,
    Qmake,
    Kks,
}

impl Display for BuildInfoKind {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{}",
            match &self {
                Self::Cargo => "cargo",
                Self::Cmake => "cmake",
                Self::Configure => "./configure",
                Self::Makefile => "make",
                Self::Install => "./install",
                Self::Jr => "./jr",
                Self::Qbs => "qbs",
                Self::Qmake => "qmake",
                Self::Kks => "kks",
            }
        )
    }
}

pub type BuildInfo = std::collections::HashSet<BuildInfoKind>;

impl SimpleBlock for BuildInfo {
    fn extend(self: Box<Self>) -> Box<dyn Pretty> {
        self
    }
}

impl From<&Environment> for BuildInfo {
    fn from(env: &Environment) -> Self {
        let workdir = &env.work_dir;
        let mut bi = BuildInfo::new();

        if file::exists("CMakeLists.txt") {
            bi.insert(BuildInfoKind::Cmake);
        }

        if file::exists("configure") {
            bi.insert(BuildInfoKind::Configure);
        }

        if file::exists("Makefile") {
            bi.insert(BuildInfoKind::Makefile);
        }

        if file::exists("install") {
            bi.insert(BuildInfoKind::Install);
        }

        if file::exists("jr") {
            bi.insert(BuildInfoKind::Jr);
        }

        if let Ok(true) = file::exists_that(workdir, |filename| filename.ends_with(".qbs")) {
            bi.insert(BuildInfoKind::Qbs);
        }

        if let Ok(true) = file::exists_that(workdir, |filename| filename.ends_with(".pro")) {
            bi.insert(BuildInfoKind::Qmake);
        }

        if file::upfind(workdir, "Cargo.toml").is_ok() {
            bi.insert(BuildInfoKind::Cargo);
        }

        if file::upfind(workdir, ".kks-workspace").is_ok() {
            bi.insert(BuildInfoKind::Kks);
        }

        bi
    }
}

impl Pretty for BuildInfo {
    fn pretty(&self, _: &IconMode) -> Option<String> {
        if self.is_empty() {
            None?
        }
        Some(
            self.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
                .boxed()
                .visible()
                .purple()
                .bold()
                .with_reset()
                .invisible()
                .to_string(),
        )
    }
}
