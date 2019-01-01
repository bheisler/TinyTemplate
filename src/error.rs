use serde_json::Error as SerdeJsonError;

#[derive(Debug)]
pub enum Error {
    ParseError { msg: String },
    RenderError { msg: String },
    SerdeError { err: SerdeJsonError },
    UnknownTemplate { msg: String },
}
impl From<SerdeJsonError> for Error {
    fn from(err: SerdeJsonError) -> Error {
        Error::SerdeError { err }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
