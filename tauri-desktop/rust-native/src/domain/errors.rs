//! Domain error types.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    DuplicateId(String),
    InvalidInput(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "duplicate id: {id}"),
            Self::InvalidInput(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for DomainError {}
