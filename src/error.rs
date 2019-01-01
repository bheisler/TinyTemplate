use instruction::{path_to_str, PathSlice};
use serde_json::Error as SerdeJsonError;
use serde_json::Value;
use std::fmt;

/// Enum representing the potential errors that TinyTemplate can encounter.
#[derive(Debug)]
pub enum Error {
    ParseError { msg: String },
    RenderError { msg: String },
    SerdeError { err: SerdeJsonError },
    FormatError { err: fmt::Error },
}
impl From<SerdeJsonError> for Error {
    fn from(err: SerdeJsonError) -> Error {
        Error::SerdeError { err }
    }
}
impl From<fmt::Error> for Error {
    fn from(err: fmt::Error) -> Error {
        Error::FormatError { err }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseError { msg } => write!(f, "Failed to parse the template. Reason: {}", msg),
            Error::RenderError { msg } => {
                write!(f, "Failed to render the template. Reason: {}", msg)
            }
            Error::SerdeError{ err } => {
                write!(f, "Unexpected serde error while converting the context to a serde_json::Value. Error: {}", err)
            }
            Error::FormatError{ err } => {
                write!(f, "Unexpected formatting error: {}", err )
            }
        }
    }
}
impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn lookup_error(step: &str, path: PathSlice, current: &Value) -> Error {
    let avail_str = if let Value::Object(object_map) = current {
        let mut avail_str = " Available values at this level are '".to_string();
        for (i, key) in object_map.keys().enumerate() {
            avail_str.push_str(key);
            if i > 0 {
                avail_str.push_str(", ");
            }
        }
        avail_str
    } else {
        "".to_string()
    };

    Error::RenderError {
        msg: format!(
            "Failed to find value '{}' from path '{}'.{}",
            step,
            path_to_str(path),
            avail_str
        ),
    }
}

pub(crate) fn truthiness_error(path: PathSlice) -> Error {
    Error::RenderError {
        msg: format!(
            "Path '{}' produced a value which could not be checked for truthiness.",
            path_to_str(path)
        ),
    }
}

pub(crate) fn unprintable_error() -> Error {
    Error::RenderError {
        msg: "Expected a printable value but found array or object.".to_string(),
    }
}

pub(crate) fn not_iterable_error(path: PathSlice) -> Error {
    Error::RenderError {
        msg: format!(
            "Expected an array for path '{}' but found a non-iterable value.",
            path_to_str(path)
        ),
    }
}

pub(crate) fn unknown_template(name: &str) -> Error {
    Error::RenderError {
        msg: format!("Tried to call an unknown template '{}'", name),
    }
}

pub(crate) fn unknown_formatter(name: &str) -> Error {
    Error::RenderError {
        msg: format!("Tried to call an unknown formatter '{}'", name),
    }
}
