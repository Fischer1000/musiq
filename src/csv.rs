pub const DEFAULT_SEPARATOR: char = ',';

/// A single value that can be represented in CSV
#[derive(Debug, Clone)]
pub enum CsvObject {
    /// A string value
    String(Box<str>),
    /// An integer value
    Int(i64),
    /// A floating-point value
    Float(f64),
    /// A boolean value
    Bool(bool),
    /// No value
    Null
}

impl CsvObject {
    /// Converts a CSV-encoded `&str` into a vector of the vectors of the values.\
    /// Each vector stores one row from the original data.\
    /// This function assumes that strings are marked with double-quotes (`"`) at both ends.
    /// # Note
    /// While usual implementations of CSV parsing don't split strings in the middle if they contain
    /// the separator character, this one does, because this use case is not needed for this project.
    pub fn from_str(s: &str, sep: char) -> Vec<Vec<CsvObject>> {
        let mut result: Vec<Vec<CsvObject>> = Vec::new();
        let mut line_buf: Vec<CsvObject> = Vec::new();

        '_lines: for line in s.lines() {
            '_values: for val in line.split(sep) {
                let val = val.trim();
                line_buf.push( match val {
                    "" | "\"" => CsvObject::Null,
                    "false" => CsvObject::Bool(false),
                    "true" => CsvObject::Bool(true),
                    _ => 'nontrivial: {
                        if let Some(remainder) = val.strip_prefix("\"") {
                            if let Some(middle) = remainder.strip_suffix("\"") {
                                break 'nontrivial CsvObject::String(middle.into());
                            }
                        }
                        if let Ok(int) = val.parse::<i64>() {
                            break 'nontrivial CsvObject::Int(int);
                        }
                        if let Ok(float) = val.parse::<f64>() {
                            break 'nontrivial CsvObject::Float(float);
                        }
                        CsvObject::Null
                    }
                } );
            }
            result.push(line_buf);
            line_buf = Vec::new();
        }

        result
    }

    /// Serializes a vector of the vectors of CSV values into a single string.\
    /// The separator is applied between values with no whitespace around it.\
    /// The rows are separated by the CRLF sequence (`\r\n`).
    pub fn serialize(values: Vec<Vec<CsvObject>>, sep: char) -> String {
        let mut result = String::new();

        for line in values {
            for (i, val) in line.iter().enumerate() {
                result.push_str( match val {
                    CsvObject::Null => String::new(),
                    CsvObject::String(s) => format!("\"{s}\""),
                    CsvObject::Int(i) => format!("{i}"),
                    CsvObject::Float(f) => format!("{f}"),
                    CsvObject::Bool(b) => format!("{b}"),
                }.as_str() );
                if i < line.len() - 1 {
                    result.push(sep);
                }
            }
            result.push_str("\r\n");
        }

        result
    }

    fn repr(&self) -> String {
        match self {
            CsvObject::Null => "null".to_string(),
            CsvObject::String(s) => format!("\"{}\"", s),
            CsvObject::Int(i) => i.to_string(),
            CsvObject::Float(f) => f.to_string(),
            CsvObject::Bool(b) => b.to_string(),
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            CsvObject::String(s) => Some(s.as_ref()),
            _ => None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            CsvObject::Bool(b) => Some(*b),
            _ => None
        }
    }
}

/// Implements `From<T>` for `CsvObject` variant `String`.
macro_rules! impl_from_str {
    ($t:ty) => {
        impl From<$t> for CsvObject {
            fn from(s: $t) -> Self {
                CsvObject::String(s.into())
            }
        }
    };
}
/// Implements `From<T>` for `CsvObject` variant `Int`.
macro_rules! impl_from_int {
    ($t:ty) => {
        impl From<$t> for CsvObject {
            fn from(i: $t) -> Self {
                CsvObject::Int(i as i64)
            }
        }
    };
}
/// Implements `From<T>` for `CsvObject` variant `Float`.
macro_rules! impl_from_float {
    ($t:ty) => {
        impl From<$t> for CsvObject {
            fn from(f: $t) -> Self {
                CsvObject::Float(f as f64)
            }
        }
    };
}

// String implementations
impl_from_str!(String);
impl_from_str!(&str);
impl_from_str!(Box<str>);

// Int implementations
impl_from_int!(i8);
impl_from_int!(i16);
impl_from_int!(i32);
impl_from_int!(i64);

// Float implementations
impl_from_float!(f32);
impl_from_float!(f64);

// Bool implementation
impl From<bool> for CsvObject {
    fn from(b: bool) -> Self {
        CsvObject::Bool(b)
    }
}

// Null implementation
impl From<()> for CsvObject {
    fn from(_: ()) -> Self {
        CsvObject::Null
    }
}