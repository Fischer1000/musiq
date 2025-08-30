#![warn(absolute_paths_not_starting_with_crate, ambiguous_negative_literals, elided_lifetimes_in_paths, ffi_unwind_calls, if_let_rescope, let_underscore_drop, meta_variable_misuse, missing_debug_implementations, redundant_imports, unit_bindings, unnameable_types, unreachable_pub, variant_size_differences)]
/*#![warn(missing_docs)]*/
#![deny(keyword_idents, unsafe_op_in_unsafe_fn)]
#![forbid(deprecated_safe_2024, non_ascii_idents, unused_crate_dependencies)]

pub static SONG_FILES_DIR: &str = ".\\songs\\";

pub mod songs;
mod macros;
pub mod database;
pub mod config;
pub mod webserver;
pub mod embedded_files;
pub mod csv;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
