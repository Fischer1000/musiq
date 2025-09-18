use std::io::Write;
use std::sync::{Mutex, atomic};
use std::fs::{File, OpenOptions};
use crate::stat;

static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);
static FILE_OPEN_FAILED: atomic::AtomicBool = atomic::AtomicBool::new(false);

static LOG_FILE_PATH: &'static str = "./latest.log";

#[macro_export]
macro_rules! logln {
    ($($arg:tt)*) => {{
        let mut text = format!($($arg)*);
        text.push('\n');

        print!("{text}");
        crate::logging::write_to_log(&text);
    }};
}

#[macro_export]
macro_rules! logln_dbg {
    ($($arg:tt)*) => {{
        let mut text = format!("{}: "; stat!(str));
        text.push_str(format!($($arg)*));
        text.push('\n');

        print!("{text}");
        crate::write_to_log(&text);
    }};
}

/// Writes a buffer to the log file and silently succeed if it cannot be written to.
pub fn write_to_log(data: &str) {
    let mut guard = LOG_FILE.lock().unwrap();

    let file = match guard.as_mut() {
        Some(file) => file,
        None => if !FILE_OPEN_FAILED.load(atomic::Ordering::Relaxed) {
            let tmp = OpenOptions::new().write(true).append(true).create(true).open(LOG_FILE_PATH);
            stat!(tmp);
            match tmp {
                Ok(file) => { *guard = Some(file); guard.as_mut().unwrap() },
                Err(_) => { FILE_OPEN_FAILED.store(true, atomic::Ordering::Relaxed); return; },
            }
        } else {
            return;
        }
    };

    let _ = file.write(data.as_bytes());
}