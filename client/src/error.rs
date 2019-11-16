use std::convert::From;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Tokio(tokio::io::Error),
    PortsExhausted,
    IncorrectTransactionId,
    IncorrectAction,
    Timeout,
    Server(String),
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Tokio(ref e) => e.description(),
            Error::PortsExhausted => "no ports available",
            Error::IncorrectTransactionId => "incorrect transaction id",
            Error::IncorrectAction => "incorrect action",
            Error::Timeout => "timeout",
            Error::Server(_) => "error from server",
        }
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Tokio(ref e) => Some(e),
            Error::PortsExhausted => None,
            Error::IncorrectTransactionId => None,
            Error::IncorrectAction => None,
            Error::Timeout => None,
            Error::Server(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Tokio(ref e) => write!(f, "tokio: {}", e),
            Error::PortsExhausted => write!(f, "ports exhausted"),
            Error::IncorrectTransactionId => write!(f, "incorrect transaction id"),
            Error::IncorrectAction => write!(f, "incorrect action"),
            Error::Timeout => write!(f, "timeout"),
            Error::Server(ref s) => write!(f, "server: {}", s),
        }
    }
}

impl From<tokio::io::Error> for Error {
    fn from(err: tokio::io::Error) -> Error {
        Error::Tokio(err)
    }
}
