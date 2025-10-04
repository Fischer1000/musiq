#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    CannotBind,
    CannotSetNonblocking,
    RequestReadFailed,
    InvalidRequest,
    CannotInferLength,
    InvalidUtf8,
    UnsupportedMethod,
    BodyTooLarge,
    ConfigFileCannotBeRead,
    InvalidConfigFile,
    DatabaseDirectoryCannotBeRead,
    DatabaseFileCannotBeRead,
    InvalidDatabaseFile,
    CannotReadFile,
    CannotOpenFile,
    CannotWriteFile,
    NoTimetableFound,
    DirectoryCannotBeRead,
    CannotCopyNewFile,
    InvalidNewFileName,
    EntryCreationFailed,
    EntryAlreadyExists,
    PathCannotBeCanonicalized,
    FileCannotBeDeleted,
    InvalidCSV,
    OutputDeviceConfigCannotBeSet,
    StreamCannotBeBuilt,
    StreamCannotBePlayed,
    NoOutputDevice,
    OutputDeviceConfigCannotBeQueried,
    NoOutputDeviceConfigs,
    CannotSetExitHandler,
    ProcessInterrupted
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Error::CannotBind => "cannot bind to the given address",
            Error::CannotSetNonblocking => "cannot set non-blocking mode",
            Error::RequestReadFailed => "cannot read request",
            Error::InvalidRequest => "invalid request",
            Error::CannotInferLength => "cannot infer request body length",
            Error::InvalidUtf8 => "invalid UTF-8 data",
            Error::UnsupportedMethod => "unsupported HTTP method",
            Error::BodyTooLarge => "request body too large",
            Error::InvalidConfigFile => "invalid configuration file",
            Error::DatabaseDirectoryCannotBeRead => "cannot read database directory",
            Error::DatabaseFileCannotBeRead => "cannot read database file",
            Error::InvalidDatabaseFile => "invalid database file",
            Error::ConfigFileCannotBeRead => "cannot read config file",
            Error::CannotReadFile => "cannot read file",
            Error::CannotOpenFile => "cannot open file",
            Error::CannotWriteFile => "cannot write file",
            Error::NoTimetableFound => "no timetable found",
            Error::DirectoryCannotBeRead => "cannot read directory",
            Error::CannotCopyNewFile => "cannot copy new file",
            Error::InvalidNewFileName => "invalid new filename",
            Error::EntryCreationFailed => "cannot create entry",
            Error::EntryAlreadyExists => "entry already exists",
            Error::PathCannotBeCanonicalized => "cannot canonicalize path",
            Error::FileCannotBeDeleted => "cannot delete file",
            Error::InvalidCSV => "invalid CSV string",
            Error::OutputDeviceConfigCannotBeSet => "cannot set output device config",
            Error::StreamCannotBeBuilt => "cannot build stream",
            Error::StreamCannotBePlayed => "cannot play stream",
            Error::NoOutputDevice => "no output device",
            Error::OutputDeviceConfigCannotBeQueried => "cannot query output device config",
            Error::NoOutputDeviceConfigs => "no output device configs",
            Error::CannotSetExitHandler => "cannot set exit handler",
            Error::ProcessInterrupted => "process interrupted"
        })
    }
}

impl std::error::Error for Error {}