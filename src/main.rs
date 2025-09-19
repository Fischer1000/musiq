use musiq::songs;

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
    let port = if let Some(port) = std::env::args().skip(1).next() {
        port
    } else {
        "80".to_string()
    };

    let host_address = if cfg!(feature = "only-local") {
        format!("localhost:{}", port)
    } else {
        format!("0.0.0.0:{}", port)
    };

    match musiq::main(host_address, musiq::SONG_FILES_DIR, has_allowed_extension, musiq::CONFIG_FILE_PATH) {
        Ok(_) => unreachable!(),
        #[cfg(feature = "debug-access")]
        Err(err) => println!("Program exited with error {err:?}"),
        #[cfg(not(feature = "debug-access"))]
        Err(_err) => ()
    }
}