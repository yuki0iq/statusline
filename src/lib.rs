//! Status line for shells with ANSI escape sequences support
//!
//! This is a documentation for statusline API, use `README.md` for executable documentation
//!
//! # Example
//!
//! Default statusline as a readline prompt generator:
//! ```
//! use statusline::{default, Environment, IconMode, Style};
//!
//! let mode = IconMode::build();
//! let args = Environment::from_env::<&str>(&[]);
//! let top = default::extend(default::top(&args));
//! let bottom = default::bottom(&args);
//!
//! // Top line is not intended to use in readline-like environments
//! eprintln!("{}", default::pretty(&top, &mode));
//!
//! // But bottom line is --- because it has "invisibility"
//! print!(
//!     "{}{}",
//!     default::title(&args).invisible(),
//!     default::pretty(&bottom, &mode)
//! );
//! ```

#![feature(
    io_error_more,
    iter_next_chunk,
    let_chains,
    stdarch_x86_has_cpuid
)]

mod args;
mod block;
mod chassis;
mod icon;
mod style;
mod time;
mod virt;

pub mod default;
pub mod file;
pub mod workgroup;

pub use crate::{
    args::Environment,
    block::{BlockType, SimpleBlock},
    chassis::Chassis,
    icon::{Icon, IconMode, Pretty},
    style::{Style, Styled},
};
