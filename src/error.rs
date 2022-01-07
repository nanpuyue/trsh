use std::fmt::{self, Debug, Display, Formatter};
use std::{error, result};

pub type Result<T> = result::Result<T, Error>;

pub trait IntoError {}

pub enum Error {
    Boxed(Box<dyn error::Error + Send>),
    String(String),
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(x) => Debug::fmt(x, f),
            Self::Boxed(x) => Debug::fmt(x, f),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(x) => Display::fmt(x, f),
            Self::Boxed(x) => Display::fmt(x, f),
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::String(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::String(s.to_owned())
    }
}

impl<E: 'static + error::Error + Send + IntoError> From<E> for Error {
    fn from(e: E) -> Self {
        Self::Boxed(Box::new(e))
    }
}

impl IntoError for std::io::Error {}
impl IntoError for std::net::AddrParseError {}

impl IntoError for openssl::ssl::Error {}
impl IntoError for openssl::error::ErrorStack {}
