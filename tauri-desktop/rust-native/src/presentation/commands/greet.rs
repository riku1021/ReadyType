//! `greet` Tauri command.

use crate::application::use_cases::greet as greet_use_case;

#[tauri::command]
pub fn greet(name: String) -> Result<String, String> {
    greet_use_case::execute(&name).map_err(|error| error.to_string())
}
