use std::fmt::{self, Display, Formatter};
use std::{error, result};

use tokio::net::TcpStream;

pub type Result<T> = result::Result<T, Error>;

pub trait IntoError {}

#[derive(Debug)]
pub enum Error {
    Boxed(Box<dyn error::Error + Send>),
    String(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => s.fmt(f),
            Self::Boxed(e) => e.fmt(f),
        }
    }
}

impl<E: 'static + error::Error + Send + IntoError> From<E> for Error {
    fn from(e: E) -> Self {
        Self::Boxed(Box::new(e))
    }
}

impl IntoError for std::io::Error {}

impl IntoError for openssl::error::ErrorStack {}
impl IntoError for tokio_openssl::HandshakeError<TcpStream> {}
