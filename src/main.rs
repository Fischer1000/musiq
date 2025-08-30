use std::io::Write;
use std::path::Path;

use musiq::{database, embedded_files};
use musiq::songs;

use musiq::or_return;
use musiq::webserver;

static SUPPORTED_EXTENSIONS: &[&str] = &["mp3"];

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
    let mut database = match database::SongDatabase::from_directory_filtered(
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

    webserver::listen("localhost:7878", webserver::handle_request, database).unwrap();
}