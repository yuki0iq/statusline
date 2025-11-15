use crate::{Block, Color, Environment, IconMode, Pretty, Style, WithStyle as _, file};

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

impl Kind {
    fn as_str(&self) -> &'static str {
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
    }
}

pub struct BuildInfo(Vec<Kind>);

super::register_block!(BuildInfo);

impl Block for BuildInfo {
    fn new(environ: &Environment) -> Option<Self> {
        let mut kinds = vec![];

        for (name, kind) in [
            ("default.nix", Kind::Nix),
            ("meson.build", Kind::Meson),
            ("CMakeLists.txt", Kind::Cmake),
            ("configure", Kind::Configure),
            ("Makefile", Kind::Makefile),
            ("jr", Kind::Jr),
            ("gradle.properties", Kind::Gradle),
        ] {
            if file::points_to_file(name) {
                kinds.push(kind);
            }
        }

        for (name, kind) in [
            ("Cargo.toml", Kind::Cargo),
            ("pyproject.toml", Kind::Pyproject),
            (".kks-workspace", Kind::Kks),
        ] {
            if file::upfind(&environ.work_dir, name).is_some() {
                kinds.push(kind);
            }
        }

        (!kinds.is_empty()).then_some(Self(kinds))
    }
}

impl Pretty for BuildInfo {
    fn pretty(&self, f: &mut std::fmt::Formatter<'_>, _: IconMode) -> std::fmt::Result {
        f.with_style(Color::PURPLE, Style::empty(), |f| {
            write!(f, "[")?;
            for (idx, kind) in self.0.iter().enumerate() {
                if idx != 0 {
                    write!(f, " ")?;
                }
                write!(f, "{}", kind.as_str())?;
            }
            write!(f, "]")
        })
    }
}
