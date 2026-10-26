use std::{
    ffi::c_void,
    fs,
    path::{Path, PathBuf},
};

use crate::application::ports::app_display_name::AppDisplayNameResolver;
use quick_xml::{events::Event, Reader, XmlVersion};
use windows::{
    core::PCWSTR,
    Management::Deployment::PackageManager,
    Win32::{
        Foundation::{
            CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, HANDLE, RPC_E_CHANGED_MODE,
        },
        Globalization::{
            GetUserDefaultUILanguage, GetUserPreferredUILanguages, LocaleNameToLCID,
            MUI_LANGUAGE_NAME,
        },
        Storage::{
            FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW},
            Packaging::Appx::{
                GetApplicationUserModelId, GetPackageFullName, GetPackagePathByFullName,
                PackageFamilyNameFromFullName,
            },
        },
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        },
        System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
            PROCESS_QUERY_LIMITED_INFORMATION,
        },
        System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
    },
};

#[derive(Default)]
pub struct WindowsAppDisplayNameResolver;

impl AppDisplayNameResolver for WindowsAppDisplayNameResolver {
    fn resolve(&self, executable_name: &str) -> Option<String> {
        resolve_running_processes(executable_name).or_else(|| executable_stem(executable_name))
    }

    fn is_bundle_resolution_cacheable(&self) -> bool {
        false
    }

    fn resolve_for_process(&self, executable_name: &str, process_id: i32) -> Option<String> {
        self.resolve_for_process_with_aumid(executable_name, process_id, None)
    }

    fn resolve_for_process_with_aumid(
        &self,
        executable_name: &str,
        process_id: i32,
        app_user_model_id: Option<&str>,
    ) -> Option<String> {
        process_display_name(process_id as u32, app_user_model_id)
            .or_else(|| executable_stem(executable_name))
    }
}

fn executable_stem(executable_name: &str) -> Option<String> {
    let name = Path::new(executable_name).file_stem()?.to_str()?.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

fn resolve_running_processes(executable_name: &str) -> Option<String> {
    let executable_name = Path::new(executable_name).file_name()?.to_str()?;
    let process_ids = matching_process_ids(executable_name)?;
    if process_ids.is_empty() {
        return None;
    }
    let names: Vec<Option<String>> = process_ids
        .into_iter()
        .map(|process_id| process_display_name(process_id, None))
        .collect();
    unique_display_name(names.iter().map(Option::as_deref))
}

fn matching_process_ids(executable_name: &str) -> Option<Vec<u32>> {
    // SAFETY: this creates a read-only snapshot of process metadata.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.ok()?;
    let mut entry = PROCESSENTRY32W::default();
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
    // SAFETY: entry is initialized and writable for Process32FirstW.
    let mut has_process = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    let mut process_ids = Vec::new();
    while has_process {
        let name_length = entry
            .szExeFile
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(entry.szExeFile.len());
        let process_name = String::from_utf16_lossy(&entry.szExeFile[..name_length]);
        if process_name.eq_ignore_ascii_case(executable_name) {
            process_ids.push(entry.th32ProcessID);
        }
        // SAFETY: entry remains a valid writable output structure.
        has_process = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }
    // SAFETY: snapshot was returned by CreateToolhelp32Snapshot.
    let _ = unsafe { CloseHandle(snapshot) };
    Some(process_ids)
}

fn unique_display_name<'a>(names: impl IntoIterator<Item = Option<&'a str>>) -> Option<String> {
    let mut unique_name: Option<&str> = None;
    for name in names {
        let name = name?.trim();
        if name.is_empty() || unique_name.is_some_and(|existing| existing != name) {
            return None;
        }
        unique_name = Some(name);
    }
    unique_name.map(str::to_owned)
}

fn process_display_name(process_id: u32, app_user_model_id: Option<&str>) -> Option<String> {
    // SAFETY: this opens the process with query-only access and a valid PID.
    let process =
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) }.ok()?;
    let mut buffer = [0u16; 32_768];
    let mut length = buffer.len() as u32;
    // SAFETY: buffer is writable and length is initialized to its capacity.
    let result = unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut length,
        )
    };
    let package_name = package_app_display_name_for_process(process, app_user_model_id);
    let image_path = result
        .is_ok()
        .then(|| String::from_utf16_lossy(&buffer[..length as usize]));
    // SAFETY: process was returned by OpenProcess and is no longer needed.
    let _ = unsafe { CloseHandle(HANDLE(process.0)) };
    package_name.or_else(|| image_path.as_deref().and_then(file_description))
}

fn package_app_display_name_for_process(
    process: HANDLE,
    explicit_aumid: Option<&str>,
) -> Option<String> {
    let aumid = explicit_aumid
        .map(str::to_owned)
        .or_else(|| process_aumid(process))?;

    let mut package_length = 0;
    // SAFETY: null output requests the required buffer size.
    let result = unsafe { GetPackageFullName(process, &mut package_length, None) };
    if result != ERROR_INSUFFICIENT_BUFFER || package_length == 0 {
        return None;
    }
    let mut package_full_name = vec![0u16; package_length as usize];
    // SAFETY: buffer has the size requested by the first call.
    if unsafe {
        GetPackageFullName(
            process,
            &mut package_length,
            Some(windows::core::PWSTR(package_full_name.as_mut_ptr())),
        )
    } != ERROR_SUCCESS
    {
        return None;
    }
    let package_full_name =
        String::from_utf16_lossy(&package_full_name[..nul_terminated_len(&package_full_name)]);
    let package_family_name = package_family_name(&package_full_name)?;
    let app_id = app_id_from_aumid(&aumid, &package_family_name)?;

    let mut path_length = 0;
    let package_full_name_wide: Vec<u16> =
        package_full_name.encode_utf16().chain(Some(0)).collect();
    // SAFETY: package_full_name_wide is a NUL-terminated package identity.
    let result = unsafe {
        GetPackagePathByFullName(
            PCWSTR(package_full_name_wide.as_ptr()),
            &mut path_length,
            None,
        )
    };
    if result != ERROR_INSUFFICIENT_BUFFER || path_length == 0 {
        return None;
    }
    let mut package_path = vec![0u16; path_length as usize];
    // SAFETY: package_path has the size requested by the first call.
    if unsafe {
        GetPackagePathByFullName(
            PCWSTR(package_full_name_wide.as_ptr()),
            &mut path_length,
            Some(windows::core::PWSTR(package_path.as_mut_ptr())),
        )
    } != ERROR_SUCCESS
    {
        return None;
    }
    let package_path = String::from_utf16_lossy(&package_path[..nul_terminated_len(&package_path)]);
    let manifest_path = PathBuf::from(package_path).join("AppxManifest.xml");
    let manifest = fs::read_to_string(manifest_path).ok()?;
    let display_name = manifest_display_name(&manifest, app_id)?;
    resolved_manifest_display_name(&display_name, || app_list_display_name(&aumid))
}

fn app_list_display_name(aumid: &str) -> Option<String> {
    let _runtime = WinRtScope::new()?;
    let packages = PackageManager::new().ok()?.FindPackages().ok()?;
    let mut names = Vec::new();
    for package in packages {
        let Ok(entries) = package.GetAppListEntries() else {
            continue;
        };
        for entry in entries {
            let Ok(entry_aumid) = entry.AppUserModelId() else {
                continue;
            };
            if entry_aumid.to_string() == aumid {
                let display_name = entry.DisplayInfo().ok()?.DisplayName().ok()?.to_string();
                names.push(display_name);
            }
        }
    }
    unique_display_name(names.iter().map(|name| Some(name.as_str())))
}

struct WinRtScope {
    initialized: bool,
}

impl WinRtScope {
    fn new() -> Option<Self> {
        // SAFETY: initializes WinRT for this thread before using PackageManager.
        match unsafe { RoInitialize(RO_INIT_MULTITHREADED) } {
            Ok(()) => Some(Self { initialized: true }),
            Err(error) if error.code() == RPC_E_CHANGED_MODE => Some(Self { initialized: false }),
            Err(_) => None,
        }
    }
}

impl Drop for WinRtScope {
    fn drop(&mut self) {
        if self.initialized {
            // SAFETY: this scope balances its successful RoInitialize call.
            unsafe { RoUninitialize() };
        }
    }
}

fn process_aumid(process: HANDLE) -> Option<String> {
    let mut length = 0;
    // SAFETY: null output requests the required buffer size.
    if unsafe { GetApplicationUserModelId(process, &mut length, None) } != ERROR_INSUFFICIENT_BUFFER
        || length == 0
    {
        return None;
    }
    let mut buffer = vec![0u16; length as usize];
    // SAFETY: buffer has the size requested by the first call.
    if unsafe {
        GetApplicationUserModelId(
            process,
            &mut length,
            Some(windows::core::PWSTR(buffer.as_mut_ptr())),
        )
    } != ERROR_SUCCESS
    {
        return None;
    }
    Some(String::from_utf16_lossy(
        &buffer[..nul_terminated_len(&buffer)],
    ))
}

fn nul_terminated_len(value: &[u16]) -> usize {
    value
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(value.len())
}

fn package_family_name(package_full_name: &str) -> Option<String> {
    let package_full_name: Vec<u16> = package_full_name.encode_utf16().chain(Some(0)).collect();
    let mut length = 0;
    // SAFETY: null output requests the required buffer size.
    let result = unsafe {
        PackageFamilyNameFromFullName(PCWSTR(package_full_name.as_ptr()), &mut length, None)
    };
    if result != ERROR_INSUFFICIENT_BUFFER || length == 0 {
        return None;
    }
    let mut buffer = vec![0u16; length as usize];
    // SAFETY: buffer has the size requested by the first call.
    if unsafe {
        PackageFamilyNameFromFullName(
            PCWSTR(package_full_name.as_ptr()),
            &mut length,
            Some(windows::core::PWSTR(buffer.as_mut_ptr())),
        )
    } != ERROR_SUCCESS
    {
        return None;
    }
    Some(String::from_utf16_lossy(
        &buffer[..nul_terminated_len(&buffer)],
    ))
}

fn app_id_from_aumid<'a>(aumid: &'a str, package_family_name: &str) -> Option<&'a str> {
    let (family, app_id) = aumid.split_once('!')?;
    (family.eq_ignore_ascii_case(package_family_name) && !app_id.is_empty()).then_some(app_id)
}

fn manifest_display_name(manifest: &str, app_id: &str) -> Option<String> {
    let mut reader = Reader::from_str(manifest);
    let mut inside_target_app = false;
    let mut target_depth = 0usize;
    let mut matches = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                let name = element.local_name();
                if name.as_ref() == "Application" {
                    let id = element
                        .attributes()
                        .flatten()
                        .find(|attribute| attribute.key.local_name().as_ref() == "Id")
                        .and_then(|attribute| {
                            attribute.normalized_value(XmlVersion::default()).ok()
                        })
                        .map(|value| value.into_owned());
                    if id.as_deref() == Some(app_id) {
                        if inside_target_app {
                            return None;
                        }
                        inside_target_app = true;
                        target_depth = 1;
                    } else if inside_target_app {
                        target_depth += 1;
                    }
                } else if inside_target_app {
                    target_depth += 1;
                    if name.as_ref() == "VisualElements" {
                        if let Some(display_name) = element
                            .attributes()
                            .flatten()
                            .find(|attribute| attribute.key.local_name().as_ref() == "DisplayName")
                            .and_then(|attribute| {
                                attribute.normalized_value(XmlVersion::default()).ok()
                            })
                            .map(|value| value.trim().to_owned())
                        {
                            if !display_name.is_empty() {
                                matches.push(display_name);
                            }
                        }
                    }
                }
            }
            Ok(Event::Empty(element))
                if inside_target_app && element.local_name().as_ref() == "VisualElements" =>
            {
                if let Some(display_name) = element
                    .attributes()
                    .flatten()
                    .find(|attribute| attribute.key.local_name().as_ref() == "DisplayName")
                    .and_then(|attribute| attribute.normalized_value(XmlVersion::default()).ok())
                    .map(|value| value.trim().to_owned())
                {
                    if !display_name.is_empty() {
                        matches.push(display_name);
                    }
                }
            }
            Ok(Event::End(element)) if inside_target_app => {
                if element.local_name().as_ref() == "Application" && target_depth == 1 {
                    inside_target_app = false;
                    target_depth = 0;
                } else {
                    target_depth = target_depth.saturating_sub(1);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => return None,
            _ => {}
        }
    }

    (matches.len() == 1).then(|| matches.remove(0))
}

fn resolved_manifest_display_name(
    display_name: &str,
    resolve_resource: impl FnOnce() -> Option<String>,
) -> Option<String> {
    if display_name.starts_with("ms-resource:") {
        resolve_resource()
    } else {
        Some(display_name.to_owned())
    }
}

fn file_description(path: &str) -> Option<String> {
    let path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    // SAFETY: path is a NUL-terminated UTF-16 path.
    let size = unsafe { GetFileVersionInfoSizeW(PCWSTR(path.as_ptr()), None) };
    if size == 0 {
        return None;
    }
    let mut data = vec![0u8; size as usize];
    // SAFETY: data has the size reported by GetFileVersionInfoSizeW.
    unsafe { GetFileVersionInfoW(PCWSTR(path.as_ptr()), None, size, data.as_mut_ptr().cast()) }
        .ok()?;

    let mut translations = std::ptr::null_mut::<c_void>();
    let mut translation_bytes = 0;
    let translation_query: Vec<u16> = "\\VarFileInfo\\Translation"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // SAFETY: data is a valid version info block and the output pointers are writable.
    let translated = unsafe {
        VerQueryValueW(
            data.as_ptr().cast(),
            PCWSTR(translation_query.as_ptr()),
            &mut translations,
            &mut translation_bytes,
        )
    }
    .as_bool();
    if translated && translation_bytes >= 4 {
        let translation = unsafe {
            std::slice::from_raw_parts(
                translations.cast::<u16>(),
                (translation_bytes / 4) as usize * 2,
            )
        };
        let translation_pairs: Vec<(u16, u16)> = translation
            .chunks_exact(2)
            .map(|pair| (pair[0], pair[1]))
            .collect();
        for (language, codepage) in
            prioritize_translations(&translation_pairs, &user_preferred_language_ids())
        {
            let query = format!(
                "\\StringFileInfo\\{:04x}{:04x}\\FileDescription",
                language, codepage
            );
            if let Some(value) = version_string(&data, &query) {
                return Some(value);
            }
        }
    }
    None
}

fn user_preferred_language_ids() -> Vec<u16> {
    let mut language_count = 0;
    let mut buffer_length = 0;
    // SAFETY: the first call requests the required buffer size.
    if unsafe {
        GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut language_count,
            None,
            &mut buffer_length,
        )
    }
    .is_err()
        || buffer_length == 0
    {
        return default_user_ui_language();
    }

    let mut buffer = vec![0u16; buffer_length as usize];
    // SAFETY: buffer has the capacity reported by the sizing call.
    if unsafe {
        GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut language_count,
            Some(windows::core::PWSTR(buffer.as_mut_ptr())),
            &mut buffer_length,
        )
    }
    .is_err()
    {
        return default_user_ui_language();
    }

    buffer
        .split(|unit| *unit == 0)
        .take_while(|language| !language.is_empty())
        .filter_map(|language| {
            let language: Vec<u16> = language.iter().copied().chain(Some(0)).collect();
            // SAFETY: language is a valid NUL-terminated BCP-47 locale name.
            let locale_id = unsafe { LocaleNameToLCID(PCWSTR(language.as_ptr()), 0) };
            (locale_id != 0).then_some(locale_id as u16)
        })
        .collect()
}

fn default_user_ui_language() -> Vec<u16> {
    // SAFETY: this reads the current user's configured UI language.
    let language = unsafe { GetUserDefaultUILanguage() };
    (language != 0).then_some(language).into_iter().collect()
}

fn prioritize_translations(
    translations: &[(u16, u16)],
    preferred_languages: &[u16],
) -> Vec<(u16, u16)> {
    let mut ordered = Vec::with_capacity(translations.len());
    for preferred in preferred_languages {
        ordered.extend(
            translations
                .iter()
                .copied()
                .filter(|(language, _)| language == preferred),
        );
    }
    for translation in translations.iter().copied() {
        if !ordered.contains(&translation) {
            ordered.push(translation);
        }
    }
    ordered
}

fn version_string(data: &[u8], query: &str) -> Option<String> {
    let query: Vec<u16> = query.encode_utf16().chain(Some(0)).collect();
    let mut value = std::ptr::null_mut::<c_void>();
    let mut length = 0;
    // SAFETY: data is a valid version info block and output pointers are writable.
    let found = unsafe {
        VerQueryValueW(
            data.as_ptr().cast(),
            PCWSTR(query.as_ptr()),
            &mut value,
            &mut length,
        )
    }
    .as_bool();
    if !found || length <= 1 {
        return None;
    }
    let units = unsafe { std::slice::from_raw_parts(value.cast::<u16>(), length as usize) };
    let description = String::from_utf16_lossy(units).trim().to_owned();
    (!description.is_empty()).then_some(description)
}

#[cfg(test)]
mod tests {
    use super::{
        app_id_from_aumid, executable_stem, manifest_display_name, prioritize_translations,
        resolved_manifest_display_name, unique_display_name, WindowsAppDisplayNameResolver,
    };
    use crate::application::ports::app_display_name::AppDisplayNameResolver;

    #[test]
    fn falls_back_to_executable_stem_for_missing_description() {
        assert_eq!(
            WindowsAppDisplayNameResolver.resolve_for_process("Code.exe", -1),
            Some("Code".to_owned())
        );
    }

    #[test]
    fn rejects_empty_executable_names() {
        assert_eq!(executable_stem(".exe"), None);
    }

    #[test]
    fn rejects_missing_or_empty_application_ids() {
        assert_eq!(app_id_from_aumid("Invalid", "Package"), None);
        assert_eq!(app_id_from_aumid("Package!", "Package"), None);
        assert_eq!(app_id_from_aumid("Wrong!App", "Package"), None);
        assert_eq!(app_id_from_aumid("Package!App", "Package"), Some("App"));
    }

    #[test]
    fn selects_the_display_name_for_the_matching_manifest_application() {
        let manifest = r#"<Package xmlns:uap="urn:uap"><Applications>
          <Application Id="First"><uap:VisualElements DisplayName="First app" /></Application>
          <Application Id="Second"><uap:VisualElements DisplayName="Second app" /></Application>
        </Applications></Package>"#;
        assert_eq!(
            manifest_display_name(manifest, "Second"),
            Some("Second app".to_owned())
        );
    }

    #[test]
    fn ignores_unresolved_or_ambiguous_manifest_names() {
        let unresolved = r#"<Package><Applications><Application Id="App"><VisualElements DisplayName="ms-resource:Name" /></Application></Applications></Package>"#;
        assert_eq!(
            manifest_display_name(unresolved, "App").as_deref(),
            Some("ms-resource:Name")
        );

        let duplicate = r#"<Package><Applications><Application Id="App"><VisualElements DisplayName="One" /></Application><Application Id="App"><VisualElements DisplayName="Two" /></Application></Applications></Package>"#;
        assert_eq!(manifest_display_name(duplicate, "App"), None);
    }

    #[test]
    fn user_ui_language_translations_are_tried_first() {
        let translations = [(0x0409, 1200), (0x0411, 1200), (0x0804, 1200)];
        assert_eq!(
            prioritize_translations(&translations, &[0x0411, 0x0409]),
            [(0x0411, 1200), (0x0409, 1200), (0x0804, 1200)]
        );
    }

    #[test]
    fn app_list_resolution_requires_matching_process_names_to_agree() {
        assert_eq!(
            unique_display_name([Some("Editor"), Some("Editor")]),
            Some("Editor".to_owned())
        );
        assert_eq!(
            unique_display_name([Some("Editor"), Some("Editor Beta")]),
            None
        );
        assert_eq!(unique_display_name([Some("Editor"), None]), None);
        assert_eq!(unique_display_name([None, None]), None);
    }

    #[test]
    fn resolves_manifest_resource_references_using_the_exact_app_entry() {
        assert_eq!(
            resolved_manifest_display_name("ms-resource:AppName", || Some(
                "Localized App".to_owned()
            )),
            Some("Localized App".to_owned())
        );
        assert_eq!(
            resolved_manifest_display_name("ms-resource:AppName", || None),
            None
        );
        assert_eq!(
            resolved_manifest_display_name("Literal App", || panic!("resource resolver called")),
            Some("Literal App".to_owned())
        );
    }
}
