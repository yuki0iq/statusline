//! Status line for shells with ANSI escape sequences support
//!
//! This is a documentation for statusline API, use `README.md` for executable documentation
//!
//! # Example
//!
//! Default statusline as a readline prompt generator:
//! ```
//! use statusline::{default, Chassis, Environment, IconMode, Style};
//!
//! let mode = IconMode::build();
//! // This is only an example, replace hardcoded values with real ones
//! let args = Environment {
//!     ret_code: None,
//!     jobs_count: 0,
//!     elapsed_time: None,
//!     work_dir: "/home/meow".into(),
//!     git_tree: None,
//!     user: "meow".into(),
//!     host: "amber".into(),
//!     chassis: Chassis::Laptop,
//!     current_home: None,
//! };
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

#![feature(io_error_more, iter_next_chunk)]
#![warn(
    clippy::cargo,
    clippy::pedantic,
    // clippy::restriction
    clippy::absolute_paths,
    clippy::allow_attributes,
    clippy::as_underscore,
    clippy::assertions_on_result_states,
    clippy::cfg_not_test,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::decimal_literal_representation,
    clippy::default_numeric_fallback,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::empty_drop,
    clippy::empty_enum_variants_with_brackets,
    clippy::empty_structs_with_brackets,
    clippy::exhaustive_enums,
    clippy::fn_to_numeric_cast_any,
    clippy::format_push_string,
    clippy::get_unwrap,
    clippy::host_endian_bytes,
    clippy::if_then_some_else_none,
    clippy::infinite_loop,
    clippy::inline_asm_x86_att_syntax,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_assert_message,
    clippy::mixed_read_write_in_expression,
    clippy::multiple_unsafe_ops_per_block,
    clippy::mutex_atomic,
    clippy::needless_raw_strings,
    clippy::partial_pub_fields,
    clippy::pathbuf_init_then_push,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::redundant_type_annotations,
    clippy::renamed_function_params,
    clippy::semicolon_outside_block,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::str_to_string,
    clippy::string_lit_chars_any,
    clippy::string_to_string,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::unused_result_ok,
    clippy::unused_trait_names,
    clippy::verbose_file_reads,
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::multiple_crate_versions,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::enum_glob_use
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
    block::{Extend, Kind as BlockType},
    chassis::Chassis,
    icon::{Icon, IconMode, Pretty},
    style::{Style, Styled},
};
