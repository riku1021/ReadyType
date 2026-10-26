use crate::{
    application::ports::input_source::{AvailableInputSource, GatewayError, InputSourceGateway},
    domain::input_source::{canonicalize_language_codes_by_family, InputSourceId},
};

pub struct NativeInputSource<H> {
    pub id: InputSourceId,
    pub name: String,
    pub language_code: Option<String>,
    pub enabled: bool,
    pub selectable: bool,
    pub handle: H,
}

impl<H: Clone> Clone for NativeInputSource<H> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            language_code: self.language_code.clone(),
            enabled: self.enabled,
            selectable: self.selectable,
            handle: self.handle.clone(),
        }
    }
}

pub trait TisBackend: Send {
    type Handle: Clone + Send;

    fn list(&mut self, locale: &str) -> Result<Vec<NativeInputSource<Self::Handle>>, GatewayError>;
    fn current(&self) -> Result<InputSourceId, GatewayError>;
    fn select(&mut self, handle: &Self::Handle) -> Result<(), GatewayError>;
    fn input_method_keyboard_layout_override(&self) -> Option<InputSourceId> {
        None
    }
}

pub struct MacOsInputSourceGateway<B: TisBackend> {
    backend: B,
    cache: Vec<NativeInputSource<B::Handle>>,
    locale: String,
}

impl<B: TisBackend> MacOsInputSourceGateway<B> {
    pub fn with_backend(mut backend: B, locale: &str) -> Result<Self, GatewayError> {
        let cache = Self::load_cache(&mut backend, locale)?;
        Ok(Self {
            backend,
            cache,
            locale: locale.to_owned(),
        })
    }

    fn load_cache(
        backend: &mut B,
        locale: &str,
    ) -> Result<Vec<NativeInputSource<B::Handle>>, GatewayError> {
        Ok(backend
            .list(locale)?
            .into_iter()
            .filter(|source| source.enabled && source.selectable)
            .collect())
    }

    fn refresh(&mut self) -> Result<(), GatewayError> {
        self.cache = Self::load_cache(&mut self.backend, &self.locale)?;
        Ok(())
    }

    fn find_handle(&self, id: &InputSourceId) -> Option<B::Handle> {
        self.cache
            .iter()
            .find(|source| source.id == *id)
            .map(|source| source.handle.clone())
    }

    fn source_flags(&self, id: &InputSourceId) -> Option<(bool, bool)> {
        self.cache
            .iter()
            .find(|source| source.id == *id)
            .map(|source| (source.enabled, source.selectable))
    }

    #[cfg(test)]
    fn backend(&self) -> &B {
        &self.backend
    }
}

impl<B: TisBackend> InputSourceGateway for MacOsInputSourceGateway<B> {
    fn list(
        &mut self,
        refresh: bool,
        locale: &str,
    ) -> Result<Vec<AvailableInputSource>, GatewayError> {
        if refresh || self.locale != locale {
            self.locale = locale.to_owned();
            self.refresh()?;
        }
        let mut entries = self
            .cache
            .iter()
            .map(|source| (source.id.as_str().to_owned(), source.language_code.clone()))
            .collect::<Vec<_>>();
        canonicalize_language_codes_by_family(&mut entries);
        Ok(self
            .cache
            .iter()
            .zip(entries.into_iter().map(|(_, language_code)| language_code))
            .map(|(source, language_code)| AvailableInputSource {
                id: source.id.clone(),
                name: source.name.clone(),
                language_code,
                available: true,
            })
            .collect())
    }

    fn current(&self) -> Result<InputSourceId, GatewayError> {
        self.backend.current()
    }

    fn select(&mut self, id: &InputSourceId) -> Result<(), GatewayError> {
        // 同ソースでも再選択する。Electron はメニューだけ合い実入力が古いことがある。
        let handle = match self.find_handle(id) {
            Some(handle) => handle,
            None => {
                self.refresh()?;
                self.find_handle(id).ok_or_else(|| {
                    GatewayError::Operation(format!(
                        "input source is unavailable: {}{}",
                        id.as_str(),
                        diagnose_unavailable_suffix(id)
                    ))
                })?
            }
        };
        let flags = self.source_flags(id).unwrap_or((false, false));
        self.backend.select(&handle).map_err(|error| {
            let base = match &error {
                GatewayError::Operation(message) => message.clone(),
                other => other.to_string(),
            };
            GatewayError::Operation(format!(
                "{base}; target={} enabled={} selectCapable={}",
                id.as_str(),
                flags.0,
                flags.1
            ))
        })
    }

    fn input_method_keyboard_layout_override(&self) -> Option<InputSourceId> {
        self.backend.input_method_keyboard_layout_override()
    }
}

/// キャッシュ外（disabled 等）のとき、可能なら enabled/selectCapable を添える。
///
/// ユニットテストでは FakeTis 経路のため実 OS の `dispatch_sync_f` を呼ばない
///（メインキューが回っておらずハングするため）。
fn diagnose_unavailable_suffix(id: &InputSourceId) -> String {
    #[cfg(all(target_os = "macos", not(test)))]
    {
        use super::macos::inspect_input_source_flags;
        if let Some((enabled, selectable)) = inspect_input_source_flags(id.as_str()) {
            return format!("; enabled={enabled} selectCapable={selectable}");
        }
    }
    let _ = id;
    String::new()
}

#[cfg(target_os = "macos")]
impl MacOsInputSourceGateway<super::macos::SystemTisBackend> {
    pub fn new(locale: &str) -> Result<Self, GatewayError> {
        Self::with_backend(super::macos::SystemTisBackend, locale)
    }
}

#[derive(Default)]
pub struct UnsupportedInputSourceGateway;

impl InputSourceGateway for UnsupportedInputSourceGateway {
    fn list(
        &mut self,
        _refresh: bool,
        _locale: &str,
    ) -> Result<Vec<AvailableInputSource>, GatewayError> {
        Err(GatewayError::UnsupportedPlatform)
    }

    fn current(&self) -> Result<InputSourceId, GatewayError> {
        Err(GatewayError::UnsupportedPlatform)
    }

    fn select(&mut self, _id: &InputSourceId) -> Result<(), GatewayError> {
        Err(GatewayError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "windows")]
pub use super::windows::input_source::WindowsInputSourceGateway;

#[cfg(any(target_os = "windows", test))]
pub(crate) mod windows_id;

#[cfg(test)]
mod windows_id_tests;

#[cfg(test)]
mod tests;
