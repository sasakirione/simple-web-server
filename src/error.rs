use std::fmt;
use std::io;
use std::error::Error as StdError;

/// Custom error types for the web server
#[derive(Debug)]
pub enum Error {
    /// IO errors
    Io(io::Error),
    /// YAML parsing errors
    YamlParse(String),
    /// Configuration errors
    Config(String),
    /// HTTP request errors
    Http(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::YamlParse(msg) => write!(f, "YAML parsing error: {}", msg),
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Http(msg) => write!(f, "HTTP error: {}", msg),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

/// Result type for the web server
pub type Result<T> = std::result::Result<T, Error>;
