use std::io::Write;
use std::path::Path;

use musiq::songs;

/*
use musiq::database;
use musiq::webserver;
use musiq::config;
use musiq::embedded_files;
*/

use musiq::or_return;

static SUPPORTED_EXTENSIONS: &[&str] = &["mp3"];
// static HOST_ADDRESS: &str = "localhost:80";

fn has_allowed_extension(s: &songs::Song) -> bool {
    SUPPORTED_EXTENSIONS.contains(
        &or_return!(
            or_return!(
                s.filename().to_str(),
                false
            ).rsplit_once('.'),
            false
        ).1.to_lowercase().as_str()
    )
}

fn main() {
    /*
    let database = match database::SongDatabase::from_directory_filtered(
        Path::new(musiq::SONG_FILES_DIR),
        has_allowed_extension
    ) {
        Ok(database) => database,
        Err(e) => match e {
            database::DatabaseError::DirectoryCannotBeRead => {
                println!("The directory of the songs cannot be read. Terminating...");
                return;
            }
            _ => unreachable!()
        }
    };

    let configs = match config::Configs::read_from_file(musiq::CONFIG_FILE_PATH) {
        Ok(configs) => configs,
        Err(_) => {
            std::fs::write(musiq::CONFIG_FILE_PATH, embedded_files::CONFIG_MUSIQ).unwrap();
            eprintln!(
                "Config file cannot be read or is invalid. A default one was created at \"{}\". Terminating...",
                 musiq::CONFIG_FILE_PATH
            );
            return;
        }
    };

    webserver::start_server("localhost:7878", webserver::handle_request, database, configs).unwrap();
    */

    let port = if let Some(port) = std::env::args().skip(1).next() {
        port
    } else {
        "80".to_string()
    };

    let host_address = format!("localhost:{}", port);

    match musiq::main(host_address, musiq::SONG_FILES_DIR, has_allowed_extension, musiq::CONFIG_FILE_PATH) {
        Ok(_) => unreachable!(),
        Err(err) => println!("Program exited with error {err:?}")
    }
}