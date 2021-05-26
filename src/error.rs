use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    inner: Box<ErrorInner>,
}

struct ErrorInner {
    kind: ErrorKind,
    status: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    ConnectionLost,
    Io,
    Parse,
    Status,
}

impl Error {
    pub(crate) fn new<E>(kind: ErrorKind, status: Option<String>, source: Option<E>) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error {
            inner: Box::new(ErrorInner {
                kind,
                status: status.map(Into::into),
                source: source.map(Into::into),
            }),
        }
    }

    pub(crate) fn new_connection_lost() -> Error {
        Error::new(ErrorKind::ConnectionLost, None, None::<Error>)
    }

    pub(crate) fn new_parse<E>(source: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::new(ErrorKind::Parse, None, Some(source))
    }

    pub(crate) fn new_status(status: String) -> Error {
        Error::new(ErrorKind::Status, Some(status), None::<Error>)
    }

    pub fn kind(&self) -> ErrorKind {
        self.inner.kind
    }

    pub fn status(&self) -> Option<&str> {
        match self.inner.kind {
            ErrorKind::Status => match self.inner.status.as_ref() {
                Some(status) => Some(status),
                None => unreachable!(),
            },
            _ => None,
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("maui::Error");

        builder.field("kind", &self.inner.kind);

        if let Some(ref source) = self.inner.source {
            builder.field("source", source);
        }

        builder.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.kind {
            ErrorKind::ConnectionLost => f.write_str("connection lost")?,
            ErrorKind::Io => f.write_str("io error")?,
            ErrorKind::Parse => f.write_str("parse error")?,
            ErrorKind::Status => write!(f, "RCON status error ({})", self.status().unwrap())?,
        };

        if let Some(ref source) = self.inner.source {
            write!(f, ": {}", source)?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source.as_ref().map(|e| &**e as _)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::new(ErrorKind::Io, None, Some(e))
    }
}
