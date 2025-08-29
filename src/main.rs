/* Phase 1
use std::ffi::OsStr;
use std::path::Path;
*/

use musiq::{or_return, print_iter};
use musiq::songs::Song;
// Phase 1: use musiq::database::FileDatabase;
use musiq::config::Configs;
use musiq::webserver::{handle_connection, listen};

// static SUPPORTED_EXTENSIONS: &[&str] = &["mp4"];
static SUPPORTED_EXTENSIONS: &[&str] = &["png", "svg"];

fn predicate(s: &Song) -> bool {
    SUPPORTED_EXTENSIONS.contains(
        &or_return!(
            or_return!(
                s.filename.to_str(),
                false
            ).rsplit_once('.'),
            false
        ).1.to_lowercase().as_str()
    )
}

fn main() {
    // drop(remove_file(".\\testdir\\pfp.png"));

    /* Phase 1
    let mut database: FileDatabase<Song> = FileDatabase::from_directory_filtered(Path::new(".\\testdir"), predicate).unwrap();

    print_iter!(database.inner().iter());

    let new_file = Path::new(".\\testdir-2\\pfp.png");

    database.add_file(new_file).realize(&mut database).unwrap();

    print_iter!(database.inner().iter());

    database.remove_entry(OsStr::new("pfp.png")).realize(&mut database).unwrap();

    print_iter!(database.inner().iter());
    */

    /*
    let configs = Configs::read_from_file(".\\testdir\\db.musiq").unwrap();

    println!("{:}", configs.timetable().display());
    */

    listen("localhost:7878", handle_connection).unwrap();
}