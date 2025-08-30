use std::ffi::OsStr;
use std::path::Path;

#[derive(Debug, Eq, Clone)]
pub struct Song {
    filename: Box<OsStr>,
    enabled: bool,
}

impl Song {
    pub fn new(filename: &Path) -> Option<Self> {
        Some(Song { filename: filename.file_name()?.into(), enabled: false })
    }

    #[inline]
    pub fn filename(&self) -> &OsStr {
        &self.filename
    }

    #[inline]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    #[inline]
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    #[inline]
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

impl std::fmt::Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.filename.display())
    }
}

impl std::hash::Hash for Song {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.filename.hash(state);
    }
}

impl std::cmp::PartialEq for Song {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
    }
}