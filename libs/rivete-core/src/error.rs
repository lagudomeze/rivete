use derive_more::{Display, Error};
use std::fmt::{Debug, Display};

/// Errors that can occur in the rivet-core library.
#[derive(Debug, Display, Error)]
pub enum Error {
    /// An unspecified error occurred.
    Other,

    /// A configuration error occurred.
    #[display("config error occurred")]
    ConfigError,

    /// Duplicate initialization attempt.
    #[display("Initialization for '{_0}' has already been done.")]
    DuplicatedInit(#[error(not(source))] &'static str),
}

/// Wrapper error that flattens an underlying error into a string message.
#[derive(Debug, Display, Error)]
pub struct FlattenError {
    #[error(not(source))]
    message: String,
}

impl FlattenError {
    /// Creates a new `FlattenError` from any displayable error.
    pub fn from(error: impl Display) -> Self {
        Self {
            message: format!("{:#}", error),
        }
    }
}
