use mongodb_cursor_pagination::error::CursorError;
use std::{error, fmt, io};

/// Possible errors that can arise during parsing and creating a cursor.
#[derive(Debug)]
pub enum ServiceError {
    IoError(io::Error),
    ParseError(String),
    MongoError(mongodb::error::Error),
    ConnectionError(String),
    InvalidCursor(String),
    NotFound(String),
    Unknown(String),
}

impl From<io::Error> for ServiceError {
    fn from(err: io::Error) -> ServiceError {
        ServiceError::IoError(err)
    }
}

impl From<bson::EncoderError> for ServiceError {
    fn from(err: bson::EncoderError) -> ServiceError {
        ServiceError::ParseError(err.to_string())
    }
}

impl From<bson::DecoderError> for ServiceError {
    fn from(err: bson::DecoderError) -> ServiceError {
        ServiceError::ParseError(err.to_string())
    }
}

impl From<mongodb::error::Error> for ServiceError {
    fn from(err: mongodb::error::Error) -> ServiceError {
        ServiceError::MongoError(err)
    }
}

impl From<&str> for ServiceError {
    fn from(message: &str) -> ServiceError {
        ServiceError::Unknown(message.to_owned())
    }
}

impl From<String> for ServiceError {
    fn from(message: String) -> ServiceError {
        ServiceError::Unknown(message)
    }
}

impl From<CursorError> for ServiceError {
    fn from(err: CursorError) -> ServiceError {
        match err {
            CursorError::IoError(err) => ServiceError::IoError(err),
            CursorError::InvalidCursor(inner)
            | CursorError::Unknown(inner)
            | CursorError::InvalidId(inner) => ServiceError::InvalidCursor(inner),
        }
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServiceError::IoError(ref inner) => inner.fmt(fmt),
            ServiceError::MongoError(ref inner) => inner.fmt(fmt),
            ServiceError::InvalidCursor(ref cursor) => {
                write!(fmt, "Invalid cursor - unable to parse: {:?}", cursor)
            }
            ServiceError::ConnectionError(ref inner)
            | ServiceError::ParseError(ref inner)
            | ServiceError::NotFound(ref inner)
            | ServiceError::Unknown(ref inner) => inner.fmt(fmt),
        }
    }
}

#[allow(deprecated)]
impl error::Error for ServiceError {
    fn description(&self) -> &str {
        match *self {
            ServiceError::IoError(ref inner) => inner.description(),
            ServiceError::MongoError(ref inner) => inner.description(),
            ServiceError::InvalidCursor(_) => "Invalid cursor value",
            ServiceError::Unknown(ref inner)
            | ServiceError::ParseError(ref inner)
            | ServiceError::ConnectionError(ref inner)
            | ServiceError::NotFound(ref inner) => inner,
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            ServiceError::IoError(ref inner) => Some(inner),
            ServiceError::MongoError(ref inner) => Some(inner),
            _ => None,
        }
    }
}
