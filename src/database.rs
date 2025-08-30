use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::Path;

use crate::songs::Song;
use crate::csv::CsvObject;
use crate::{or_continue, or_return, return_unless};

#[derive(Debug)]
pub enum DatabaseError {
    DirectoryCannotBeRead,
    CannotCopyNewFile,
    InvalidNewFileName,
    EntryCreationFailed,
    EntryAlreadyExists,
    PathCannotBeCanonicalized,
    FileCannotBeDeleted
}

pub struct SongDatabase {
    root_dir: Box<Path>,
    entries: HashSet<Song>
}

impl SongDatabase {
    fn get_directory_contents<P: AsRef<Path>>(path: P) -> Result<impl Iterator<Item = Box<Path>>, DatabaseError> {
        use DatabaseError::DirectoryCannotBeRead;

        Ok(read_dir(&path)
            .map_err(|_| DirectoryCannotBeRead)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                !or_return!(e.metadata().ok(), false)
                    .is_dir()
            })
            .map(|e| e.path().into_boxed_path())
        )
    }

    pub fn from_directory<P: AsRef<Path> + Clone>(path: P) -> Result<Self, DatabaseError>
    where Box<Path>: From<P>
    {
        Self::from_vec(
            Self::get_directory_contents(&path)?.filter_map(|p| Song::new(p.as_ref())).collect(),
            path.clone()
        )
    }

    pub fn from_directory_filtered<P, F>(path: P, predicate: F) -> Result<Self, DatabaseError>
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

    pub fn from_vec<P: AsRef<Path>>(entries: HashSet<Song>, root_dir: P) -> Result<Self, DatabaseError>
    where Box<Path>: From<P>
    {
        Ok(Self { entries, root_dir: or_return!(root_dir.as_ref().canonicalize().ok(), Err(DatabaseError::PathCannotBeCanonicalized)).into() })
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
        &self.entries
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut HashSet<Song> {
        &mut self.entries
    }

    pub fn refresh<F: FnMut(&Song) -> bool>(&mut self, predicate: F) -> Result<(), DatabaseError> {
        *self = Self::from_directory_filtered(self.root_dir.as_ref(), predicate)?;
        Ok(())
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn get_songs_css(&self) -> Vec<Vec<CsvObject>> {
        let mut result = Vec::new();

        for (filename, enabled) in self.entries.iter().map(|s| (s.filename(), s.enabled())) {
            let filename = or_continue!(filename.to_str()).into();
            let enabled = enabled.into();

            result.push(vec![filename, enabled]);
        }

        result
    }
}

#[derive(Debug)]
pub enum DatabaseTransaction {
    EntryAdded { file_path: Box<Path> },
    EntryRemoved { file_name: Box<OsStr> }
}

#[allow(unreachable_patterns)]
impl DatabaseTransaction {
    pub fn realize(self, database: &mut SongDatabase, file_ops: bool) -> Result<(), (DatabaseTransaction, DatabaseError)> {
        use DatabaseTransaction::*;
        use DatabaseError::*;

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
                    database.entries.insert(or_return!(
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

                database.entries.remove(&or_return!(
                    Song::new(file_path.as_ref()),
                    Err((self, EntryCreationFailed))
                ));
            }
            _ => todo!()
        };

        Ok(())
    }
}