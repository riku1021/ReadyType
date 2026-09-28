use std::fmt;

use crate::domain::input_source::InputSourceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableInputSource {
    pub id: InputSourceId,
    pub name: String,
    /// BCP 47 language code from TIS (`kTISPropertyInputSourceLanguages` の先頭)。無い場合は `None`。
    pub language_code: Option<String>,
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayError {
    UnsupportedPlatform,
    Operation(String),
}

impl fmt::Display for GatewayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(formatter, "input sources are unsupported"),
            Self::Operation(message) => {
                write!(formatter, "input source operation failed: {message}")
            }
        }
    }
}

impl std::error::Error for GatewayError {}

pub trait InputSourceGateway: Send {
    fn list(&mut self, refresh: bool) -> Result<Vec<AvailableInputSource>, GatewayError>;
    fn current(&self) -> Result<InputSourceId, GatewayError>;
    fn select(&mut self, id: &InputSourceId) -> Result<(), GatewayError>;
}

impl<T: InputSourceGateway + ?Sized> InputSourceGateway for Box<T> {
    fn list(&mut self, refresh: bool) -> Result<Vec<AvailableInputSource>, GatewayError> {
        (**self).list(refresh)
    }

    fn current(&self) -> Result<InputSourceId, GatewayError> {
        (**self).current()
    }

    fn select(&mut self, id: &InputSourceId) -> Result<(), GatewayError> {
        (**self).select(id)
    }
}
