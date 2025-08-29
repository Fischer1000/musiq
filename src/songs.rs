use std::ffi::OsStr;
use std::path::Path;

use crate::database::Entry;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Song {
    // title: Box<str>,
    // artist: Box<str>,
    pub filename: Box<OsStr>,
}

impl Song {
    pub fn new(filename: &Path) -> Option<Self> {
        Some(Song { filename: filename.file_name()?.into() })
    }

    #[inline]
    pub fn filename(&self) -> &OsStr {
        &self.filename
    }
}

impl Entry<Box<Path>> for Song {
    fn create_entry(data: Box<Path>) -> Option<Song> {
        Song::new(data.as_ref())
    }

    fn into_data(self) -> Box<Path> {
        Path::new(self.filename.as_ref()).into()
    }
}

impl std::fmt::Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.filename.display())
    }
}