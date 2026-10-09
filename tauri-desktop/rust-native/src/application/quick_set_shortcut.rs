//! Quick Set グローバルショートカット（Tauri accelerator 文字列）の正規化。

use crate::domain::errors::DomainError;

/// 既定: macOS は ⌘⇧L、Windows / Linux は Ctrl+Shift+L。
pub const DEFAULT_QUICK_SET_SHORTCUT: &str = "CommandOrControl+Shift+L";

const MODIFIER_ORDER: &[&str] = &[
    "CommandOrControl",
    "Command",
    "Super",
    "Control",
    "Alt",
    "Shift",
];

/// 入力を正規化した accelerator（例: `CommandOrControl+Shift+L`）にする。
pub fn normalize_quick_set_shortcut(raw: &str) -> Result<String, DomainError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(DomainError::InvalidInput(
            "quick set shortcut must not be empty".to_owned(),
        ));
    }

    let parts: Vec<&str> = trimmed
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() < 2 {
        return Err(DomainError::InvalidInput(
            "quick set shortcut requires a modifier and a key".to_owned(),
        ));
    }

    let key_raw = parts[parts.len() - 1];
    let key = normalize_key(key_raw)
        .ok_or_else(|| DomainError::InvalidInput(format!("unsupported shortcut key: {key_raw}")))?;

    let mut modifiers = Vec::new();
    for part in &parts[..parts.len() - 1] {
        let modifier = normalize_modifier(part).ok_or_else(|| {
            DomainError::InvalidInput(format!("unsupported shortcut modifier: {part}"))
        })?;
        if !modifiers.contains(&modifier) {
            modifiers.push(modifier);
        }
    }
    if modifiers.is_empty() {
        return Err(DomainError::InvalidInput(
            "quick set shortcut requires a modifier".to_owned(),
        ));
    }

    modifiers.sort_by_key(|modifier| {
        MODIFIER_ORDER
            .iter()
            .position(|ordered| *ordered == *modifier)
            .unwrap_or(MODIFIER_ORDER.len())
    });

    let mut tokens = modifiers;
    tokens.push(key.as_str());
    Ok(tokens.join("+"))
}

fn normalize_modifier(part: &str) -> Option<&'static str> {
    match part.to_ascii_lowercase().as_str() {
        "commandorcontrol" | "cmdorctrl" | "cmdorcontrol" => Some("CommandOrControl"),
        "control" | "ctrl" => Some("Control"),
        "alt" | "option" => Some("Alt"),
        "shift" => Some("Shift"),
        "super" | "win" | "meta" => Some("Super"),
        "command" | "cmd" => Some("Command"),
        _ => None,
    }
}

fn normalize_key(part: &str) -> Option<String> {
    if part.len() == 1 {
        let character = part.chars().next()?;
        if character.is_ascii_alphanumeric() {
            return Some(character.to_ascii_uppercase().to_string());
        }
    }
    match part.to_ascii_lowercase().as_str() {
        "space" => Some("Space".to_owned()),
        "tab" => Some("Tab".to_owned()),
        "enter" | "return" => Some("Enter".to_owned()),
        "escape" | "esc" => Some("Esc".to_owned()),
        "backspace" => Some("Backspace".to_owned()),
        "up" | "arrowup" => Some("ArrowUp".to_owned()),
        "down" | "arrowdown" => Some("ArrowDown".to_owned()),
        "left" | "arrowleft" => Some("ArrowLeft".to_owned()),
        "right" | "arrowright" => Some("ArrowRight".to_owned()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_default_shortcut() {
        assert_eq!(
            normalize_quick_set_shortcut("commandorcontrol+shift+l").unwrap(),
            DEFAULT_QUICK_SET_SHORTCUT
        );
    }

    #[test]
    fn rejects_key_only() {
        assert!(normalize_quick_set_shortcut("L").is_err());
    }

    #[test]
    fn accepts_control_alt_shift() {
        assert_eq!(
            normalize_quick_set_shortcut("ctrl+alt+shift+a").unwrap(),
            "Control+Alt+Shift+A"
        );
    }
}
