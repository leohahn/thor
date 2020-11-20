use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error("unexpected end of input")]
    Eof,

    #[error("invalid syntax")]
    Syntax,

    #[error("trailing characters")]
    TrailingCharacters,

    #[error("expected integer")]
    ExpectedInteger,

    #[error("expected end of integer")]
    ExpectedIntegerEnd,

    #[error("expected byte string")]
    ExpectedByteString,

    #[error("expected char")]
    ExpectedChar,

    #[error("expected string")]
    ExpectedString,

    #[error("expected list")]
    ExpectedList,

    #[error("expected map")]
    ExpectedMap,

    #[error("expected end of map")]
    ExpectedMapEnd,

    #[error("expected end of array")]
    ExpectedArrayEnd,

    #[error("expected enum")]
    ExpectedEnum,

    #[error("expected list or bytes")]
    ExpectedSequence,
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
