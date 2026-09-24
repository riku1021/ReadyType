//! Guest name value object used by the greet use case.

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuestName(String);

impl GuestName {
    pub fn parse(raw: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(raw.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
