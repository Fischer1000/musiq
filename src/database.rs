use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::Path;

use crate::{or_return, return_unless};

pub trait Entry<T> where Self: Sized {
    fn create_entry(data: T) -> Option<Self>;
    fn into_data(self) -> T;
}

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

pub struct FileDatabase<T: Eq + std::hash::Hash> {
    root_dir: Box<Path>,
    // entries: Vec<T>,
    entries: HashSet<T>
}

impl<T: Entry<Box<Path>> + Eq + std::hash::Hash> FileDatabase<T> {
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
            Self::get_directory_contents(&path)?.filter_map(T::create_entry).collect(),
            path.clone()
        )
    }

    pub fn from_directory_filtered<P, F>(path: P, predicate: F) -> Result<Self, DatabaseError>
    where P: AsRef<Path> + Clone, F: FnMut(&T) -> bool, Box<Path>: From<P>
    {
        Self::from_vec(
            Self::get_directory_contents(&path)?
                .filter_map(T::create_entry)
                .filter(predicate)
                .collect(),
            path.clone()
        )
    }

    pub fn from_vec<P: AsRef<Path>>(entries: HashSet<T>, root_dir: P) -> Result<Self, DatabaseError>
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
    pub fn inner(&self) -> &HashSet<T> {
        &self.entries
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut HashSet<T> {
        &mut self.entries
    }
}

#[derive(Debug)]
pub enum DatabaseTransaction {
    EntryAdded { file_path: Box<Path> },
    EntryRemoved { file_name: Box<OsStr> }
}

#[allow(unreachable_patterns)]
impl DatabaseTransaction {
    pub fn realize<T: Entry<Box<Path>> + Eq + std::hash::Hash>(self, database: &mut FileDatabase<T>) -> Result<(), (DatabaseTransaction, DatabaseError)> {
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

                or_return!(
                    std::fs::copy(&file_path, &new_path).ok(),
                    Err((self, CannotCopyNewFile))
                );
                return_unless!(
                    database.entries.insert(or_return!(
                        T::create_entry(new_path.into_boxed_path()),
                        Err((self, EntryCreationFailed))
                    )),
                    Err((self, EntryAlreadyExists))
                );
            }
            EntryRemoved { file_name } => {
                let file_path = database.root_dir.join(file_name.as_ref());

                match std::fs::remove_file(&file_path) {
                    Ok(_) => (),
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::NotFound => (),
                        _ => return Err((self, FileCannotBeDeleted))
                    }
                }

                database.entries.remove(&or_return!(
                    T::create_entry(file_path.into_boxed_path()),
                    Err((self, EntryCreationFailed))
                ));
            }
            _ => todo!()
        };

        Ok(())
    }
}