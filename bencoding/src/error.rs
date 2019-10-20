use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
    Eof,
    Syntax,
    TrailingCharacters,
    ExpectedInteger,
    ExpectedIntegerEnd,
    ExpectedByteString,
    ExpectedChar,
    ExpectedString,
    ExpectedList,
    ExpectedMap,
    ExpectedMapEnd,
    ExpectedArrayEnd,
    ExpectedEnum,
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

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(std::error::Error::description(self))
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            Error::Eof => "unexpected end of input",
            Error::Syntax => "invalid syntax",
            Error::TrailingCharacters => "trailing characters",
            Error::ExpectedInteger => "expected integer",
            Error::ExpectedIntegerEnd => "expected end of integer",
            Error::ExpectedByteString => "expected byte string",
            Error::ExpectedChar => "expected char",
            Error::ExpectedString => "expected string",
            Error::ExpectedList => "expected list",
            Error::ExpectedMap => "expected map",
            Error::ExpectedMapEnd => "expected end of map",
            Error::ExpectedArrayEnd => "expected end of array",
            Error::ExpectedEnum => "expected enum",
            Error::ExpectedSequence => "expected list or bytes",
        }
    }
}
