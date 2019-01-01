use instruction::{path_to_str, PathSlice};
use serde_json::Error as SerdeJsonError;
use serde_json::Value;
use std::fmt::Error as FormatError;

#[derive(Debug)]
pub enum Error {
    ParseError { msg: String },
    RenderError { msg: String },
    SerdeError { err: SerdeJsonError },
    FormatError { err: FormatError },
    UnknownTemplate { msg: String },
}
impl From<SerdeJsonError> for Error {
    fn from(err: SerdeJsonError) -> Error {
        Error::SerdeError { err }
    }
}
impl From<FormatError> for Error {
    fn from(err: FormatError) -> Error {
        Error::FormatError { err }
    }
}

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
