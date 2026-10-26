use windows::{
    core::{Error, Interface, GUID},
    Win32::{
        Foundation::S_FALSE,
        Globalization::{GetLocaleInfoW, LCIDToLocaleName},
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_MULTITHREADED,
        },
        UI::{
            Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardLayoutList, HKL},
            TextServices::{
                CLSID_TF_InputProcessorProfiles, ITfInputProcessorProfileMgr,
                ITfInputProcessorProfiles, GUID_TFCAT_TIP_KEYBOARD, TF_INPUTPROCESSORPROFILE,
                TF_IPPMF_FORSESSION, TF_IPP_FLAG_ENABLED, TF_PROFILETYPE_INPUTPROCESSOR,
                TF_PROFILETYPE_KEYBOARDLAYOUT,
            },
            WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
        },
    },
};

use crate::{
    application::ports::input_source::{AvailableInputSource, GatewayError, InputSourceGateway},
    domain::input_source::{InputSourceId, LanguageTag},
    infrastructure::platform::input_source::windows_id::{
        activate_profile_hresult_succeeded, format_windows_source_id, merge_windows_language_ids,
        parse_windows_source_id, WindowsSourceId,
    },
};

const LOCALE_SLANGUAGE: u32 = 0x0000_0002;

pub struct WindowsInputSourceGateway {
    cache: Vec<AvailableInputSource>,
    loaded: bool,
}

impl WindowsInputSourceGateway {
    pub fn new() -> Self {
        Self {
            cache: Vec::new(),
            loaded: false,
        }
    }

    fn refresh(&mut self) -> Result<(), GatewayError> {
        self.cache = enumerate_sources()?;
        self.loaded = true;
        Ok(())
    }
}

impl InputSourceGateway for WindowsInputSourceGateway {
    fn list(
        &mut self,
        refresh: bool,
        _locale: &str,
    ) -> Result<Vec<AvailableInputSource>, GatewayError> {
        if refresh || !self.loaded {
            self.refresh()?;
        }
        Ok(self.cache.clone())
    }

    fn current(&self) -> Result<InputSourceId, GatewayError> {
        let _com = ComScope::new()?;
        let manager = profile_manager()?;
        let mut profile = TF_INPUTPROCESSORPROFILE::default();
        // SAFETY: the profile manager is a live COM object and `profile` is writable.
        if unsafe { manager.GetActiveProfile(&GUID_TFCAT_TIP_KEYBOARD, &mut profile) }.is_ok() {
            let source = source_from_profile(&profile);
            return input_source_id(&source);
        }

        let foreground = unsafe { GetForegroundWindow() };
        let thread_id = unsafe { GetWindowThreadProcessId(foreground, None) };
        let hkl = unsafe { GetKeyboardLayout(thread_id) };
        input_source_id(&WindowsSourceId::KeyboardLayout(hkl_to_u64(hkl)))
    }

    fn select(&mut self, id: &InputSourceId) -> Result<(), GatewayError> {
        let Some(source) = parse_windows_source_id(id.as_str()) else {
            return Err(GatewayError::Operation(format!(
                "invalid Windows input source id: {}",
                id.as_str()
            )));
        };
        let _com = ComScope::new()?;
        let manager = profile_manager()?;
        let hresult = match source {
            WindowsSourceId::KeyboardLayout(raw_hkl) => {
                let hkl = HKL(raw_hkl as usize as *mut _);
                // SAFETY: TSF validates the registered HKL and GUID_NULL marks a keyboard layout.
                unsafe {
                    (Interface::vtable(&manager).ActivateProfile)(
                        Interface::as_raw(&manager),
                        TF_PROFILETYPE_KEYBOARDLAYOUT,
                        (raw_hkl & 0xffff) as u16,
                        &GUID::zeroed(),
                        &GUID::zeroed(),
                        hkl,
                        TF_IPPMF_FORSESSION,
                    )
                }
            }
            WindowsSourceId::TextService {
                language_id,
                class_id,
                profile_id,
            } => {
                let class_id = GUID::try_from(class_id.as_str()).map_err(operation_error)?;
                let profile_id = GUID::try_from(profile_id.as_str()).map_err(operation_error)?;
                // SAFETY: GUIDs and LANGID came from a validated, enumerated TSF source id.
                unsafe {
                    (Interface::vtable(&manager).ActivateProfile)(
                        Interface::as_raw(&manager),
                        TF_PROFILETYPE_INPUTPROCESSOR,
                        language_id,
                        &class_id,
                        &profile_id,
                        HKL::default(),
                        TF_IPPMF_FORSESSION,
                    )
                }
            }
        };
        if activate_profile_hresult_succeeded(hresult.0) {
            Ok(())
        } else if hresult == S_FALSE {
            Err(GatewayError::Operation(
                "Windows input source profile is not enabled".to_owned(),
            ))
        } else {
            Err(operation_error(Error::from_hresult(hresult)))
        }
    }
}

fn enumerate_sources() -> Result<Vec<AvailableInputSource>, GatewayError> {
    let _com = ComScope::new()?;
    let profile_manager = profile_manager()?;
    let profiles = profiles()?;
    let mut layouts = Vec::new();
    // SAFETY: passing None asks Windows for the number of keyboard layouts.
    let count = unsafe { GetKeyboardLayoutList(None) };
    if count > 0 {
        layouts.resize(count as usize, HKL::default());
        // SAFETY: the slice has the capacity returned by the preceding call.
        let written = unsafe { GetKeyboardLayoutList(Some(&mut layouts)) };
        layouts.truncate(written.max(0) as usize);
    }

    let mut sources = Vec::new();
    let mut layout_language_ids = Vec::new();
    for hkl in layouts {
        let raw_hkl = hkl_to_u64(hkl);
        let language_id = (raw_hkl & 0xffff) as u16;
        layout_language_ids.push(language_id);
        sources.push(available_source(
            WindowsSourceId::KeyboardLayout(raw_hkl),
            locale_display_name(language_id).unwrap_or_else(|| format!("Input {language_id:04X}")),
            language_tag(language_id),
        )?);
    }

    let language_ids =
        merge_windows_language_ids(&layout_language_ids, &tsf_language_ids(&profiles)?);
    for language_id in language_ids {
        // SAFETY: EnumProfiles returns the registered profiles for the supplied LANGID.
        let enumerator =
            unsafe { profile_manager.EnumProfiles(language_id) }.map_err(operation_error)?;
        loop {
            let mut profile = [TF_INPUTPROCESSORPROFILE::default()];
            let mut fetched = 0;
            // SAFETY: `profile` has one writable slot and `fetched` is writable.
            match unsafe { enumerator.Next(&mut profile, &mut fetched) } {
                Ok(()) if fetched > 0 => {}
                Ok(()) => break,
                Err(error) => return Err(operation_error(error)),
            }
            let profile = profile[0];
            if profile.dwProfileType != TF_PROFILETYPE_INPUTPROCESSOR
                || profile.dwFlags & TF_IPP_FLAG_ENABLED == 0
            {
                continue;
            }
            // SAFETY: profile identifiers were returned by the live TSF enumerator.
            let name = unsafe {
                profiles.GetLanguageProfileDescription(
                    &profile.clsid,
                    profile.langid,
                    &profile.guidProfile,
                )
            }
            .ok()
            .map(|value| value.to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| {
                format!(
                    "{} IME",
                    locale_display_name(profile.langid).unwrap_or_default()
                )
            });
            sources.push(available_source(
                source_from_profile(&profile),
                name,
                language_tag(profile.langid),
            )?);
        }
    }
    sources.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.id.as_str().cmp(right.id.as_str()))
    });
    sources.dedup_by(|left, right| left.id == right.id);
    Ok(sources)
}

fn tsf_language_ids(profiles: &ITfInputProcessorProfiles) -> Result<Vec<u16>, GatewayError> {
    let mut raw_language_ids = std::ptr::null_mut();
    let mut count = 0;
    // SAFETY: both output pointers are writable; TSF allocates the returned array.
    unsafe { profiles.GetLanguageList(&mut raw_language_ids, &mut count) }
        .map_err(operation_error)?;
    if raw_language_ids.is_null() || count == 0 {
        if !raw_language_ids.is_null() {
            // SAFETY: TSF allocated this array even though it contained no entries.
            unsafe { CoTaskMemFree(Some(raw_language_ids.cast())) };
        }
        return Ok(Vec::new());
    }
    // SAFETY: TSF guarantees `count` readable LANGIDs in its allocated output array.
    let language_ids =
        unsafe { std::slice::from_raw_parts(raw_language_ids, count as usize) }.to_vec();
    // SAFETY: GetLanguageList allocated this array with CoTaskMemAlloc.
    unsafe { CoTaskMemFree(Some(raw_language_ids.cast())) };
    Ok(language_ids)
}

fn available_source(
    key: WindowsSourceId,
    name: String,
    language_code: Option<String>,
) -> Result<AvailableInputSource, GatewayError> {
    Ok(AvailableInputSource {
        id: InputSourceId::parse(format_windows_source_id(&key)).map_err(operation_error)?,
        name,
        language_code,
        available: true,
    })
}

fn input_source_id(source: &WindowsSourceId) -> Result<InputSourceId, GatewayError> {
    InputSourceId::parse(format_windows_source_id(source)).map_err(operation_error)
}

fn source_from_profile(profile: &TF_INPUTPROCESSORPROFILE) -> WindowsSourceId {
    if profile.dwProfileType == TF_PROFILETYPE_INPUTPROCESSOR {
        WindowsSourceId::TextService {
            language_id: profile.langid,
            class_id: guid_string(profile.clsid),
            profile_id: guid_string(profile.guidProfile),
        }
    } else {
        WindowsSourceId::KeyboardLayout(hkl_to_u64(profile.hkl))
    }
}

fn guid_string(guid: GUID) -> String {
    format!(
        "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        guid.data1,
        guid.data2,
        guid.data3,
        guid.data4[0],
        guid.data4[1],
        guid.data4[2],
        guid.data4[3],
        guid.data4[4],
        guid.data4[5],
        guid.data4[6],
        guid.data4[7],
    )
}

fn profile_manager() -> Result<ITfInputProcessorProfileMgr, GatewayError> {
    // SAFETY: CLSID and interface type are the documented TSF profile manager.
    unsafe {
        CoCreateInstance::<_, ITfInputProcessorProfileMgr>(
            &CLSID_TF_InputProcessorProfiles,
            None,
            CLSCTX_INPROC_SERVER,
        )
    }
    .map_err(operation_error)
}

fn profiles() -> Result<ITfInputProcessorProfiles, GatewayError> {
    // SAFETY: CLSID and interface type are the documented TSF profile collection.
    unsafe {
        CoCreateInstance::<_, ITfInputProcessorProfiles>(
            &CLSID_TF_InputProcessorProfiles,
            None,
            CLSCTX_INPROC_SERVER,
        )
    }
    .map_err(operation_error)
}

fn locale_display_name(language_id: u16) -> Option<String> {
    let mut buffer = [0u16; 128];
    // SAFETY: `buffer` is writable and `language_id` is a Windows LANGID/LCID value.
    let written =
        unsafe { GetLocaleInfoW(language_id as u32, LOCALE_SLANGUAGE, Some(&mut buffer)) };
    (written > 1).then(|| String::from_utf16_lossy(&buffer[..written as usize - 1]))
}

fn language_tag(language_id: u16) -> Option<String> {
    let mut buffer = [0u16; 85];
    // SAFETY: `buffer` is the documented writable locale-name output.
    let written = unsafe { LCIDToLocaleName(language_id as u32, Some(&mut buffer), 0) };
    if written <= 1 {
        return None;
    }
    LanguageTag::parse(&String::from_utf16_lossy(&buffer[..written as usize - 1]))
        .ok()
        .map(|tag| tag.as_str().to_owned())
}

fn hkl_to_u64(hkl: HKL) -> u64 {
    hkl.0 as usize as u64
}

fn operation_error(error: impl std::fmt::Display) -> GatewayError {
    GatewayError::Operation(error.to_string())
}

struct ComScope {
    initialized: bool,
}

impl ComScope {
    fn new() -> Result<Self, GatewayError> {
        // SAFETY: each scope balances a successful COM initialization on this thread.
        let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if result.is_ok() {
            Ok(Self { initialized: true })
        } else if result.0 as u32 == 0x8001_0106 {
            // The Tauri thread may already be initialized as STA; it still has a usable COM apartment.
            Ok(Self { initialized: false })
        } else {
            Err(GatewayError::Operation(format!(
                "CoInitializeEx failed with HRESULT {}",
                result.0
            )))
        }
    }
}

impl Drop for ComScope {
    fn drop(&mut self) {
        if self.initialized {
            // SAFETY: this instance exists only after a successful CoInitializeEx call.
            unsafe { CoUninitialize() };
        }
    }
}
