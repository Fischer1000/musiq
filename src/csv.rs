use std::fs::File;
use std::io::{BufRead, BufReader, Error as IoError, Write};

pub struct CSVFile {
    file: File,
    has_header: bool,
}

impl<const N: usize> CSVFile {
    /// Creates a new `CSVFile` object from a given path by opening the file there.\
    /// # Note:
    /// This function does not check for whether the file's extension is `csv` and assumes the file
    /// does not contain a header specifying field names.
    pub fn new(path: &str) -> Result<CSVFile, IoError> {
        let file = File::open(path)?;

        Ok(CSVFile { file, has_header: false })
    }

    /// Informs the object that the opened file has a header specifying field names.
    pub fn set_header(&mut self) {
        self.has_header = true;
    }

    /// Informs the object that the opened file does not have a header specifying field names.
    pub fn unset_header(&mut self) {
        self.has_header = false;
    }

    /// Iterates over the lines of the underlying file and parses them into a single value
    /// by calling the specified function on each line read.
    ///
    /// # Usage
    /// ```
    /// # use musiq::csv::CSVFile;
    /// # compile_error!("This test should not be runned under no circumstances,
    /// # because of its possible use of the compiling machine's files.");
    /// // The file is expected to have two values in each row
    ///
    /// fn to_ints(line: &str) -> Option<(i32, i32)> {
    ///     let (first, second) = {
    ///         let mut i = line.split(',');
    ///         (i.next()?, i.next()?)
    ///     };
    ///     Some((
    ///         first.parse().ok()?,
    ///         second.parse().ok()?
    ///     ))
    /// }
    ///
    /// let my_csv_file = CSVFile::new("foo.csv").unwrap();
    ///
    /// let collected = my_csv_file.collect_values(to_ints);
    /// ```
    pub fn collect_values<T>(mut self, f: fn(&str) -> Option<T>) -> Vec<T> {
        let mut lines = BufReader::new(self.file)
            .lines()
            .skip(usize::from_bool(self.has_header)); // Skip the first line if there is a header

        lines.filter_map(|v| f(v.ok()?.as_str())).collect::<Vec<T>>()
    }

    pub fn add_entry(&mut self, entry: &str) -> std::io::Result<()> {
        self.file.write_all(entry.as_bytes())
    }

    pub fn from_entries<const N: usize>(filename: &str, entries: Vec<&[&str; N]>) -> Result<CSVFile, ()> {
        todo!();
        let file = File::create(filename.into());
    }
}