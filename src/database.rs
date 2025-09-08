use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::Path;

use crate::songs::Song;
use crate::csv::CsvObject;
use crate::{csv, or_continue, or_return, return_unless, stat};
use crate::Error;

/*
#[derive(Debug)]
pub enum Error {
    DirectoryCannotBeRead,
    CannotCopyNewFile,
    InvalidNewFileName,
    EntryCreationFailed,
    EntryAlreadyExists,
    PathCannotBeCanonicalized,
    FileCannotBeDeleted,
    InvalidCSV
}
*/

pub struct SongDatabase {
    root_dir: Box<Path>,
    songs: HashSet<Song>
}

impl SongDatabase {
    fn get_directory_contents<P: AsRef<Path>>(path: P) -> Result<impl Iterator<Item = Box<Path>>, Error> {
        Ok(read_dir(&path)
            .map_err(|_| Error::DirectoryCannotBeRead)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                !or_return!(e.metadata().ok(), false)
                    .is_dir()
            })
            .map(|e| e.path().into_boxed_path())
        )
    }

    pub fn from_directory<P: AsRef<Path> + Clone>(path: P) -> Result<Self, Error>
    where Box<Path>: From<P>
    {
        Self::from_vec(
            Self::get_directory_contents(&path)?.filter_map(|p| Song::new(p.as_ref())).collect(),
            path.clone()
        )
    }

    pub fn from_directory_filtered<P, F>(path: P, predicate: F) -> Result<Self, Error>
    where P: AsRef<Path> + Clone, F: FnMut(&Song) -> bool, Box<Path>: From<P>
    {
        Self::from_vec(
            Self::get_directory_contents(&path)?
                .filter_map(|p| Song::new(p.as_ref()))
                .filter(predicate)
                .collect(),
            path.clone()
        )
    }

    pub fn from_vec<P: AsRef<Path>>(entries: HashSet<Song>, root_dir: P) -> Result<Self, Error>
    where Box<Path>: From<P>
    {
        Ok(Self { songs: entries, root_dir: or_return!(root_dir.as_ref().canonicalize().ok(), Err(Error::PathCannotBeCanonicalized)).into() })
    }

    #[inline]
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> DatabaseTransaction
    where Box<Path>: From<P>
    {
        DatabaseTransaction::EntryAdded { file_path: path.into() }
    }

    #[inline]
    pub fn remove_entry(&mut self, entry_name: &OsStr) -> DatabaseTransaction {
        DatabaseTransaction::EntryRemoved { file_name: entry_name.into() }
    }

    #[inline]
    pub fn inner(&self) -> &HashSet<Song> {
        &self.songs
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut HashSet<Song> {
        &mut self.songs
    }

    pub fn refresh<F: FnMut(&Song) -> bool>(&mut self, predicate: F) -> Result<(), Error> {
        *self = Self::from_directory_filtered(self.root_dir.as_ref(), predicate)?;
        Ok(())
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn get_songs_csv(&self) -> Vec<Vec<CsvObject>> {
        let mut result = Vec::new();

        let mut entries = self.songs.iter().map(|s| (s.filename(), s.enabled())).collect::<Vec<_>>();
        entries.sort_by(|(f0, _), (f1, _)| f0.cmp(f1));

        for (filename, enabled) in entries {
            let filename = or_continue!(filename.to_str()).into();
            let enabled = enabled.into();

            result.push(vec![filename, enabled]);
        }

        result
    }

    /// Updates this database with the contents of the given CSV
    /// and returns the number of valid entries added or changed.
    pub fn update_from_csv(&mut self, entries: Vec<Vec<CsvObject>>) -> Result<usize, Error> {
        let mut added: usize = 0;

        for entry in entries {
            let [filename, enabled]: [CsvObject; 2] = or_return!(entry.try_into().ok(), Err(Error::InvalidCSV));

            let filename = Path::new( or_return!(
                filename.as_string(),
                Err(Error::InvalidCSV)
            ));

            if !self.root_dir.join(filename).exists() { continue; }

            let mut song = or_return!(
                Song::new(filename),
                Err(Error::InvalidCSV)
            );

            if or_return!(enabled.as_bool(), Err(Error::InvalidCSV)) {
                song.enable();
            } else {
                song.disable();
            }

            self.songs.replace(song);

            added += 1;
        }

        Ok(added)
    }

    pub fn reset_played(&mut self) {
        let songs = self
            .songs
            .iter()
            .map(|song| {
                let mut s = song.clone();
                s.set_played(false);
                s
            })
            .collect::<HashSet<_>>();
        self.songs = songs;
    }
}

#[derive(Debug)]
pub enum DatabaseTransaction {
    EntryAdded { file_path: Box<Path> },
    EntryRemoved { file_name: Box<OsStr> }
}

#[allow(unreachable_patterns)]
impl DatabaseTransaction {
    pub fn realize(self, database: &mut SongDatabase, file_ops: bool) -> Result<(), (DatabaseTransaction, Error)> {
        use DatabaseTransaction::*;
        use Error::*;

        match &self {
            EntryAdded { file_path } => {
                let file_name = or_return!(
                    file_path.file_name(),
                    Err((self, InvalidNewFileName))
                );
                let new_path = database.root_dir.join(file_name);

                // println!("file_path: {}\nfile_name: {}\nnew_path: {}", file_path.display(), file_name.display(), new_path.display());

                if file_ops {
                    or_return!(
                        std::fs::copy(&file_path, &new_path).ok(),
                        Err((self, CannotCopyNewFile))
                    );
                }

                return_unless!(
                    database.songs.insert(or_return!(
                        Song::new(new_path.as_ref()),
                        Err((self, EntryCreationFailed))
                    )),
                    Err((self, EntryAlreadyExists))
                );
            }
            EntryRemoved { file_name } => {
                let file_path = database.root_dir.join(file_name.as_ref());

                if file_ops {
                    match std::fs::remove_file(&file_path) {
                        Ok(_) => (),
                        Err(e) => match e.kind() {
                            std::io::ErrorKind::NotFound => (),
                            _ => return Err((self, FileCannotBeDeleted))
                        }
                    }
                }

                database.songs.remove(&or_return!(
                    Song::new(file_path.as_ref()),
                    Err((self, EntryCreationFailed))
                ));
            }
            _ => todo!()
        };

        Ok(())
    }
}