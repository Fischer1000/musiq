#![warn(
    absolute_paths_not_starting_with_crate, ambiguous_negative_literals, elided_lifetimes_in_paths,
    ffi_unwind_calls, if_let_rescope, meta_variable_misuse, redundant_imports, unit_bindings,
    unnameable_types, unreachable_pub, variant_size_differences
)]
// #![warn(missing_docs, missing_debug_implementations)]
#![deny(keyword_idents, unsafe_op_in_unsafe_fn, unexpected_cfgs)]
#![forbid(deprecated_safe_2024, non_ascii_idents, unused_crate_dependencies)]

// #![feature(vec_into_raw_parts)]

use std::convert::Infallible;
use std::io::Write;
use std::net::{TcpListener, ToSocketAddrs};
use std::path::Path;
use std::time::SystemTime;
use cpal::traits::HostTrait;
use crate::time::{Time, Day};

pub mod songs;
mod macros;
pub mod database;
pub mod config;
pub mod webserver;
pub mod csv;
pub mod time;
pub mod logging;
pub mod events;
mod error;

pub mod generated { include!(concat!(env!("OUT_DIR"), "/generated.rs")); }

pub use crate::error::Error;
use crate::events::EventQueue;

/// The package version from `Cargo.toml`
pub static VERSION: &str = env!("CARGO_PKG_VERSION");
pub static SONG_FILES_DIR: &str = "./songs/";
pub static EVENT_FILES_DIR: &str = "./events/";
pub static CONFIG_FILE_PATH: &str = "./config.musiq";
pub static DATABASE_FILE_NAME: &str = "db.csv";
pub static EVENT_QUEUE_FILE_NAME: &str = "queue.bin";
pub const PLAYLIST_LENGTH: usize = 1;
pub const TIMEOUT: Option<core::time::Duration> = Some(core::time::Duration::from_secs(1));

/// Sets up the program and runs the main loop,
/// which then calls handles for TCP requests, and handles time-related events.
pub fn main<A: ToSocketAddrs, P: AsRef<Path> + Clone + Send + 'static, F: FnMut(&songs::Song) -> bool>(
    addr: A,
    database_path: P,
    database_filter: F,
    config_file_path: P,
    event_files_path: P
) -> Result<Infallible, Error> {
    logln!("Started in version: {}", VERSION);

    let listener = TcpListener::bind(&addr).map_err(|_| Error::CannotBind)?;
    listener.set_nonblocking(true).map_err(|_| Error::CannotSetNonblocking)?;

    let database_path = database_path.as_ref();
    let database_file_name = database_path.join(DATABASE_FILE_NAME);

    let _ = std::fs::create_dir(SONG_FILES_DIR);
    let _ = std::fs::create_dir(EVENT_FILES_DIR);

    let mut database = match database::SongDatabase::from_directory_filtered(database_path, database_filter) {
        Ok(database) => database,
        Err(e) => {
            match e {
                Error::DirectoryCannotBeRead => {
                    eprintln!("The directory of the songs cannot be read.\nTerminating...");
                }
                _ => eprintln!("Unexpected error while trying to read the songs directory.\nTerminating...")
            }
            return Err(Error::DatabaseDirectoryCannotBeRead);
        }
    };

    'database_file: {
        or_return!(
            database.update_from_csv(
                csv::CsvObject::from_str(
                    or_return!(
                        str::from_utf8(
                            or!(
                                std::fs::read(&database_file_name).ok(),
                                break 'database_file
                            ).as_slice()
                        ).ok(),
                        Err(Error::InvalidDatabaseFile)),
                    csv::DEFAULT_SEPARATOR,
                    csv::DEFAULT_STR_MARKER
                )
            ).ok(),
            Err(Error::InvalidDatabaseFile)
        );
    }

    let mut configs = match config::Configs::read_from_file(&config_file_path) {
        Ok(configs) => configs,
        Err(e) => {
            match e {
                Error::CannotReadFile => {
                    std::fs::write(&config_file_path, &config::default_config_bytes()).unwrap();
                    config::Configs::from_bytes(&config::default_config_bytes(), &config_file_path).unwrap()
                },
                _ => {
                    eprintln!("Config file is invalid.\nTerminating...");
                    return Err(Error::InvalidConfigFile);
                }
            }
        }
    };

    let mut event_queue = EventQueue::load_from_file(
        event_files_path
            .as_ref()
            .join(EVENT_QUEUE_FILE_NAME)
    )?;

    let mut play_thread: Option<std::thread::JoinHandle<_>> = None;

    #[allow(unused_labels)]
    '_main: loop {
        if let Ok((mut stream, _)) = listener.accept() {
            stream.set_nonblocking(false).map_err(|_| Error::CannotSetNonblocking)?;

            let _ = stream.set_read_timeout(TIMEOUT);
            let _ = stream.set_write_timeout(TIMEOUT);

            let response = webserver::handle_request(
                webserver::Request::from_stream(&stream),
                &mut database,
                &mut configs,
                &mut event_queue,
            );

            let _ = stream.write_all(response.as_bytes().as_slice());

            let _ = configs.save_to_file(&config_file_path);
            
            let _ = database.save_to_file();

            let _ = event_queue.save_to_file(event_files_path.as_ref().join(EVENT_QUEUE_FILE_NAME));
        }

        let timestamp = { // TODO: separate this into a function
            let mut timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("The system time is before the UNIX epoch.")
                .as_secs();

            let offset = configs.utc_offset() as i64;

            match offset {
                i64::MIN..0 => timestamp -= offset.abs() as u64 * 3600,
                0..=i64::MAX => timestamp += offset as u64 * 3600
            }

            timestamp
        };

        let now = Time::now(configs.utc_offset());

        let events_to_trigger = event_queue.trigger_events(timestamp);

        let helper = |configs: &mut config::Configs, database: &mut database::SongDatabase| {
            if let Some(action) = configs.timetable().action(
                &now,
                &Day::today(configs.utc_offset())
            ) {
                if !action { return None; }
                let playlist = or_return!(songs::compose_playlist(PLAYLIST_LENGTH, database), None);

                logln!("Scheduled play started at {}", now);
                Some(std::thread::spawn(move || songs::play_playlist(&playlist)))
            } else if events_to_trigger.len() > 0 {
                let event_files_path = event_files_path.clone();

                // TODO: make this into a separate function in events.rs
                Some(std::thread::spawn(move || {
                    let host = cpal::default_host();
                    let device = or_return!(host.default_output_device(), Err(Error::NoOutputDevice));

                    logln!("Scheduled event started at {}", now);
                    for event in events_to_trigger {
                        let path = event_files_path.as_ref().join(event.obtain_filename().as_ref());

                        songs::play_mp3(
                            &path,
                            &device,
                            |rms, scale_factor, duration_secs|
                                logln!(
                                    "Playing \"{}\" ({:.1} seconds, RMS = {rms}, α={scale_factor})",
                                    event.name(),
                                    duration_secs
                                )
                        )?
                    }

                    Ok(())
                }))
            } else {
                None
            }
        };

        match &play_thread {
            Some(t) if t.is_finished() => play_thread = helper(&mut configs, &mut database),
            None => play_thread = helper(&mut configs, &mut database),
            Some(_) => ()
        }
    }
}

pub fn enable_all<P: AsRef<Path> + Clone, F: FnMut(&songs::Song) -> bool>(
    database_path: P,
    database_filter: F,
) -> Result<(), Error> {
    logln!("Enabling all songs...");

    let database_path = database_path.as_ref();
    let database_file_name = database_path.join(DATABASE_FILE_NAME);

    let _ = std::fs::create_dir(SONG_FILES_DIR);

    let mut database = match database::SongDatabase::from_directory_filtered(database_path, database_filter) {
        Ok(database) => database,
        Err(e) => {
            match e {
                Error::DirectoryCannotBeRead => {
                    eprintln!("The directory of the songs cannot be read.\nTerminating...");
                }
                _ => eprintln!("Unexpected error while trying to read the songs directory.\nTerminating...")
            }
            return Err(Error::DatabaseDirectoryCannotBeRead);
        }
    };

    or_return!(
        database.update_from_csv(
            csv::CsvObject::from_str(
                or_return!(
                    str::from_utf8(
                        or_return!(
                            std::fs::read(&database_file_name).ok(),
                            Ok(())
                        ).as_slice()
                    ).ok(),
                    Err(Error::InvalidDatabaseFile)),
                csv::DEFAULT_SEPARATOR,
                csv::DEFAULT_STR_MARKER
            )
        ).ok(),
        Err(Error::InvalidDatabaseFile)
    );

    database.enable_all();

    Ok(())
}

pub fn disable_all<P: AsRef<Path> + Clone, F: FnMut(&songs::Song) -> bool>(
    database_path: P,
    database_filter: F,
) -> Result<(), Error> {
    logln!("Disabling all songs...");

    let database_path = database_path.as_ref();
    let database_file_name = database_path.join(DATABASE_FILE_NAME);

    let _ = std::fs::create_dir(SONG_FILES_DIR);

    let mut database = match database::SongDatabase::from_directory_filtered(database_path, database_filter) {
        Ok(database) => database,
        Err(e) => {
            match e {
                Error::DirectoryCannotBeRead => {
                    eprintln!("The directory of the songs cannot be read.\nTerminating...");
                }
                _ => eprintln!("Unexpected error while trying to read the songs directory.\nTerminating...")
            }
            return Err(Error::DatabaseDirectoryCannotBeRead);
        }
    };

    or_return!(
        database.update_from_csv(
            csv::CsvObject::from_str(
                or_return!(
                    str::from_utf8(
                        or_return!(
                            std::fs::read(&database_file_name).ok(),
                            Ok(())
                        ).as_slice()
                    ).ok(),
                    Err(Error::InvalidDatabaseFile)),
                csv::DEFAULT_SEPARATOR,
                csv::DEFAULT_STR_MARKER
            )
        ).ok(),
        Err(Error::InvalidDatabaseFile)
    );

    database.disable_all();

    Ok(())
}

pub fn reset_played<P: AsRef<Path> + Clone, F: FnMut(&songs::Song) -> bool>(
    database_path: P,
    database_filter: F,
) -> Result<(), Error> {
    logln!("Resetting played status of all songs...");

    let database_path = database_path.as_ref();
    let database_file_name = database_path.join(DATABASE_FILE_NAME);

    let _ = std::fs::create_dir(SONG_FILES_DIR);

    let mut database = match database::SongDatabase::from_directory_filtered(database_path, database_filter) {
        Ok(database) => database,
        Err(e) => {
            match e {
                Error::DirectoryCannotBeRead => {
                    eprintln!("The directory of the songs cannot be read.\nTerminating...");
                }
                _ => eprintln!("Unexpected error while trying to read the songs directory.\nTerminating...")
            }
            return Err(Error::DatabaseDirectoryCannotBeRead);
        }
    };

    or_return!(
        database.update_from_csv(
            csv::CsvObject::from_str(
                or_return!(
                    str::from_utf8(
                        or_return!(
                            std::fs::read(&database_file_name).ok(),
                            Ok(())
                        ).as_slice()
                    ).ok(),
                    Err(Error::InvalidDatabaseFile)),
                csv::DEFAULT_SEPARATOR,
                csv::DEFAULT_STR_MARKER
            )
        ).ok(),
        Err(Error::InvalidDatabaseFile)
    );

    database.reset_played();

    Ok(())
}

/*
#[cfg(test)]
mod tests {
    use super::*;
}
*/
