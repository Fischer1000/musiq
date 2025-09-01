#![warn(absolute_paths_not_starting_with_crate, ambiguous_negative_literals, elided_lifetimes_in_paths, ffi_unwind_calls, if_let_rescope, let_underscore_drop, meta_variable_misuse, missing_debug_implementations, redundant_imports, unit_bindings, unnameable_types, unreachable_pub, variant_size_differences)]
/*#![warn(missing_docs)]*/
#![deny(keyword_idents, unsafe_op_in_unsafe_fn)]
#![forbid(deprecated_safe_2024, non_ascii_idents, unused_crate_dependencies)]

use std::convert::Infallible;
use std::io::Write;
use std::net::{TcpListener, ToSocketAddrs};

use crate::time::{Time, Day};

pub static SONG_FILES_DIR: &str = ".\\songs\\";
pub static CONFIG_FILE_PATH: &str = ".\\config.musiq";

pub mod songs;
mod macros;
pub mod database;
pub mod config;
pub mod webserver;
pub mod embedded_files;
pub mod csv;
pub mod time;

/// Runs the main loop, which then calls handles for TCP requests, and handles time-related events.
pub fn main_loop<A: ToSocketAddrs>(
    addr: A,
    mut database: database::SongDatabase,
    mut configs: config::Configs
) -> Result<Infallible, webserver::Error> {
    let listener = TcpListener::bind(addr).map_err(|_| webserver::Error::CannotBind)?;
    listener.set_nonblocking(true).map_err(|_| webserver::Error::CannotSetNonblocking)?;

    loop {
        if let Ok((mut stream, _)) = listener.accept() {
            let response = webserver::handle_request(
                webserver::Request::from_stream(&stream),
                &mut database,
                &mut configs,
            );

            let _ = stream.write_all(response.as_bytes().as_slice());
        };

        'action: { if let Some(action) = configs.timetable().action(
            &Time::now(configs.utc_offset()),
            &Day::today(configs.utc_offset())
        ) {
            if !action { break 'action }
            let playlist = or!(songs::compose_playlist(1, &database), break 'action);

            let _ = std::thread::spawn(move || { songs::play(&playlist) });
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
