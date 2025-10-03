use musiq::{songs, generated};

use musiq::or_return;

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
    let debug_mode = std::env::var("DEBUG").as_deref().unwrap_or("false") == "true";

    let host_address = if let Some(port) = std::env::args().skip(1).next() {
        port
    } else {
        "0.0.0.0:80".to_string()
    };

    if debug_mode { println!("Generated directory: {}", generated::GENERATED_DIRECTORY); }

    match musiq::main(host_address, musiq::SONG_FILES_DIR, has_allowed_extension, musiq::CONFIG_FILE_PATH) {
        Ok(_) => unreachable!(),
        Err(err) => if debug_mode { println!("Program exited with error {err:?}"); }
    }
}