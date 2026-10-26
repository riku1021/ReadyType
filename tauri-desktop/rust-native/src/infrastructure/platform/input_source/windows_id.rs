#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WindowsSourceId {
    KeyboardLayout(u64),
    TextService {
        language_id: u16,
        class_id: String,
        profile_id: String,
    },
}

pub(crate) fn merge_windows_language_ids(layout_ids: &[u16], tsf_ids: &[u16]) -> Vec<u16> {
    let mut language_ids = Vec::new();
    for language_id in layout_ids.iter().chain(tsf_ids) {
        if !language_ids.contains(language_id) {
            language_ids.push(*language_id);
        }
    }
    language_ids
}

pub(crate) fn activate_profile_hresult_succeeded(hresult: i32) -> bool {
    hresult == 0
}

pub(crate) fn format_windows_source_id(source: &WindowsSourceId) -> String {
    match source {
        WindowsSourceId::KeyboardLayout(hkl) => format!("win:hkl:{hkl:016X}"),
        WindowsSourceId::TextService {
            language_id,
            class_id,
            profile_id,
        } => format!(
            "win:tsf:{language_id:04X}:{}:{}",
            class_id.to_ascii_uppercase(),
            profile_id.to_ascii_uppercase()
        ),
    }
}

pub(crate) fn parse_windows_source_id(value: &str) -> Option<WindowsSourceId> {
    let mut parts = value.split(':');
    match (parts.next()?, parts.next()?) {
        ("win", "hkl") => {
            let raw_hkl = parts.next()?;
            if parts.next().is_some() || raw_hkl.len() != 16 {
                return None;
            }
            u64::from_str_radix(raw_hkl, 16)
                .ok()
                .map(WindowsSourceId::KeyboardLayout)
        }
        ("win", "tsf") => {
            let raw_language_id = parts.next()?;
            let raw_class_id = parts.next()?;
            let raw_profile_id = parts.next()?;
            if parts.next().is_some() || raw_language_id.len() != 4 {
                return None;
            }
            let language_id = u16::from_str_radix(raw_language_id, 16).ok()?;
            if !is_guid(raw_class_id) || !is_guid(raw_profile_id) {
                return None;
            }
            Some(WindowsSourceId::TextService {
                language_id,
                class_id: raw_class_id.to_ascii_uppercase(),
                profile_id: raw_profile_id.to_ascii_uppercase(),
            })
        }
        _ => None,
    }
}

fn is_guid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        })
}
