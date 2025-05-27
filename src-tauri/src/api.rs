use std::fmt::{Debug, Display};

use serde::{Serialize, ser::Serializer};

pub mod auth;

use crate::BoxError;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    source: BoxError,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.source, f)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.source, f)
    }
}

impl std::error::Error for Error {}

impl From<BoxError> for Error {
    fn from(source: BoxError) -> Self {
        Self { source }
    }
}

impl From<String> for Error {
    fn from(source: String) -> Self {
        Self {
            source: BoxError::from(source),
        }
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
