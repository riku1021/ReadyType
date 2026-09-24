//! Greet use case: builds a greeting message from a guest name.

use crate::domain::entities::guest_name::GuestName;
use crate::domain::errors::DomainError;

pub fn execute(name: &str) -> Result<String, DomainError> {
    let guest = GuestName::parse(name)?;
    Ok(format!(
        "Hello, {}! You've been greeted from Rust!",
        guest.as_str()
    ))
}
