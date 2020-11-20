#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tokio: {0}")]
    Tokio(#[from] tokio::io::Error),

    #[error("ports exhausted")]
    PortsExhausted,

    #[error("incorrect transaction id")]
    IncorrectTransactionId,

    #[error("incorrect auction")]
    IncorrectAction,

    #[error("timeout")]
    Timeout,

    #[error("addr parsing: {0}")]
    AddrParsing(#[from] std::net::AddrParseError),

    #[error("server: {0}")]
    Server(String),
}
