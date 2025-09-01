extern crate rand;

use rand::{rng, seq::SliceRandom};
use std::ffi::OsStr;
use std::path::Path;

use crate::database::SongDatabase;

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

    pub fn play(&self) {
        todo!()
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

/// Composes a playlist with the given number of elements from a database's songs.
pub fn compose_playlist(elem_cnt: usize, database: &SongDatabase) -> Option<Vec<Song>> {
    let mut elems: Vec<_> = database
        .inner()
        .iter()
        .filter_map(|e| if e.enabled { Some(e.clone()) } else { None })
        .collect();

    if elem_cnt < elems.len() { return None; }

    elems.shuffle(&mut rng());

    elems.truncate(elem_cnt);

    Some(elems)
}

/// Plays each song in a playlist sequentially.
/// # Warning
/// This function blocks its thread while the songs are playing.
pub fn play(playlist: &[Song]) {
    for song in playlist {
        song.play();
    }
}