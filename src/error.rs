#[derive(Debug)]
pub enum Error {
    ParseError{msg: String},
}
pub type Result<T> = std::result::Result<T, Error>;