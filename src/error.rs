use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    inner: Box<ErrorInner>,
}

struct ErrorInner {
    kind: ErrorKind,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    ConnectionLost,
    Io,
    Parse,
    Status(String),
}

impl Error {
    pub(crate) fn new<E>(kind: ErrorKind, source: Option<E>) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error {
            inner: Box::new(ErrorInner {
                kind,
                source: source.map(Into::into),
            }),
        }
    }

    pub(crate) fn new_connection_lost() -> Error {
        Error::new(ErrorKind::ConnectionLost, None::<Error>)
    }

    pub(crate) fn new_parse<E>(source: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::new(ErrorKind::Parse, Some(source))
    }

    pub(crate) fn new_status(code: String) -> Error {
        Error::new(ErrorKind::Status(code), None::<Error>)
    }

    pub fn is_status(&self) -> bool {
        matches!(self.inner.kind, ErrorKind::Status(_))
    }

    pub fn status(&self) -> Option<&str> {
        match self.inner.kind {
            ErrorKind::Status(ref code) => Some(code),
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
            ErrorKind::Status(ref code) => write!(f, "RCON status error ({})", code)?,
        };

        if let Some(ref source) = self.inner.source {
            write!(f, ": {}", source)?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::new(ErrorKind::Io, Some(e))
    }
}
