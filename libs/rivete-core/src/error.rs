use derive_more::{Display, Error};
use std::fmt::{Debug, Display};

#[derive(Debug, Display, Error)]
pub enum Error {
    Other,

    #[display("config error occurred")]
    ConfigError,

    #[display("Initialization for '{_0}' has already been done.")]
    DuplicatedInit(#[error(not(source))] &'static str),
}

#[derive(Debug, Display, Error)]
pub struct FlattenError {
    #[error(not(source))]
    message: String,
}

impl FlattenError {
    pub fn from(error: impl Display) -> Self {
        Self {
            message: format!("{:#}", error),
        }
    }
}
