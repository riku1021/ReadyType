use crate::{
    application::ports::input_source::{AvailableInputSource, GatewayError, InputSourceGateway},
    domain::input_source::InputSourceId,
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

    fn list(&mut self) -> Result<Vec<NativeInputSource<Self::Handle>>, GatewayError>;
    fn current(&self) -> Result<InputSourceId, GatewayError>;
    fn select(&mut self, handle: &Self::Handle) -> Result<(), GatewayError>;
}

pub struct MacOsInputSourceGateway<B: TisBackend> {
    backend: B,
    cache: Vec<NativeInputSource<B::Handle>>,
}

impl<B: TisBackend> MacOsInputSourceGateway<B> {
    pub fn with_backend(mut backend: B) -> Result<Self, GatewayError> {
        let cache = Self::load_cache(&mut backend)?;
        Ok(Self { backend, cache })
    }

    fn load_cache(backend: &mut B) -> Result<Vec<NativeInputSource<B::Handle>>, GatewayError> {
        Ok(backend
            .list()?
            .into_iter()
            .filter(|source| source.enabled && source.selectable)
            .collect())
    }

    fn refresh(&mut self) -> Result<(), GatewayError> {
        self.cache = Self::load_cache(&mut self.backend)?;
        Ok(())
    }

    fn find_handle(&self, id: &InputSourceId) -> Option<B::Handle> {
        self.cache
            .iter()
            .find(|source| source.id == *id)
            .map(|source| source.handle.clone())
    }

    #[cfg(test)]
    fn backend(&self) -> &B {
        &self.backend
    }
}

impl<B: TisBackend> InputSourceGateway for MacOsInputSourceGateway<B> {
    fn list(&mut self, refresh: bool) -> Result<Vec<AvailableInputSource>, GatewayError> {
        if refresh {
            self.refresh()?;
        }
        Ok(self
            .cache
            .iter()
            .map(|source| AvailableInputSource {
                id: source.id.clone(),
                name: source.name.clone(),
                language_code: source.language_code.clone(),
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
                    GatewayError::Operation(format!("input source is unavailable: {}", id.as_str()))
                })?
            }
        };
        self.backend.select(&handle)
    }
}

#[cfg(target_os = "macos")]
impl MacOsInputSourceGateway<super::macos::SystemTisBackend> {
    pub fn new() -> Result<Self, GatewayError> {
        Self::with_backend(super::macos::SystemTisBackend)
    }
}

#[derive(Default)]
pub struct UnsupportedInputSourceGateway;

impl InputSourceGateway for UnsupportedInputSourceGateway {
    fn list(&mut self, _refresh: bool) -> Result<Vec<AvailableInputSource>, GatewayError> {
        Err(GatewayError::UnsupportedPlatform)
    }

    fn current(&self) -> Result<InputSourceId, GatewayError> {
        Err(GatewayError::UnsupportedPlatform)
    }

    fn select(&mut self, _id: &InputSourceId) -> Result<(), GatewayError> {
        Err(GatewayError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::{MacOsInputSourceGateway, NativeInputSource, TisBackend};
    use crate::{
        application::ports::input_source::{GatewayError, InputSourceGateway},
        domain::input_source::InputSourceId,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct FakeHandle(u8);

    struct FakeTis {
        lists: VecDeque<Result<Vec<NativeInputSource<FakeHandle>>, GatewayError>>,
        current: InputSourceId,
        list_calls: usize,
        select_calls: usize,
        selection_error: Option<GatewayError>,
    }

    impl FakeTis {
        fn with_lists(lists: Vec<Vec<NativeInputSource<FakeHandle>>>) -> Self {
            Self {
                lists: lists.into_iter().map(Ok).collect(),
                current: input_id("en"),
                list_calls: 0,
                select_calls: 0,
                selection_error: None,
            }
        }
    }

    impl TisBackend for FakeTis {
        type Handle = FakeHandle;

        fn list(&mut self) -> Result<Vec<NativeInputSource<Self::Handle>>, GatewayError> {
            self.list_calls += 1;
            self.lists.pop_front().unwrap_or_else(|| Ok(Vec::new()))
        }

        fn current(&self) -> Result<InputSourceId, GatewayError> {
            Ok(self.current.clone())
        }

        fn select(&mut self, _handle: &Self::Handle) -> Result<(), GatewayError> {
            self.select_calls += 1;
            if let Some(error) = &self.selection_error {
                return Err(error.clone());
            }
            Ok(())
        }
    }

    fn input_id(value: &str) -> InputSourceId {
        InputSourceId::parse(value).expect("valid input source")
    }

    fn source(
        id: &str,
        name: &str,
        enabled: bool,
        selectable: bool,
        handle: u8,
    ) -> NativeInputSource<FakeHandle> {
        NativeInputSource {
            id: input_id(id),
            name: name.to_owned(),
            language_code: None,
            enabled,
            selectable,
            handle: FakeHandle(handle),
        }
    }

    #[test]
    fn missing_target_refreshes_once_then_selects() {
        let backend = FakeTis::with_lists(vec![
            vec![source("en", "ABC", true, true, 1)],
            vec![
                source("en", "ABC", true, true, 1),
                source("jp", "Japanese", true, true, 2),
            ],
        ]);
        let mut gateway = MacOsInputSourceGateway::with_backend(backend).expect("initial list");

        gateway
            .select(&input_id("jp"))
            .expect("select after refresh");

        assert_eq!(gateway.backend().list_calls, 2);
        assert_eq!(gateway.backend().select_calls, 1);
    }

    #[test]
    fn list_exposes_names_and_filters_disabled_or_unselectable_sources() {
        let backend = FakeTis::with_lists(vec![vec![
            source("en", "ABC", true, true, 1),
            source("disabled", "Disabled", false, true, 2),
            source("mode", "Mode", true, false, 3),
        ]]);
        let mut gateway = MacOsInputSourceGateway::with_backend(backend).expect("initial list");

        let available = gateway.list(false).expect("cached list");

        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id.as_str(), "en");
        assert_eq!(available[0].name, "ABC");
        assert!(available[0].available);
    }

    #[test]
    fn selecting_current_source_still_calls_native_select() {
        let backend = FakeTis::with_lists(vec![vec![source("en", "ABC", true, true, 1)]]);
        let mut gateway = MacOsInputSourceGateway::with_backend(backend).expect("initial list");

        gateway.select(&input_id("en")).expect("already selected");

        assert_eq!(gateway.backend().select_calls, 1);
    }

    #[test]
    fn native_selection_error_is_propagated() {
        let mut backend = FakeTis::with_lists(vec![vec![source("jp", "Japanese", true, true, 2)]]);
        backend.selection_error = Some(GatewayError::Operation("OSStatus -50".to_owned()));
        let mut gateway = MacOsInputSourceGateway::with_backend(backend).expect("initial list");

        let error = gateway
            .select(&input_id("jp"))
            .expect_err("selection fails");

        assert_eq!(error, GatewayError::Operation("OSStatus -50".to_owned()));
        assert_eq!(gateway.backend().select_calls, 1);
    }

    #[test]
    fn absent_target_after_refresh_fails_without_repeated_refreshes() {
        let backend = FakeTis::with_lists(vec![
            vec![source("en", "ABC", true, true, 1)],
            vec![source("en", "ABC", true, true, 1)],
        ]);
        let mut gateway = MacOsInputSourceGateway::with_backend(backend).expect("initial list");

        assert!(gateway.select(&input_id("jp")).is_err());
        assert_eq!(gateway.backend().list_calls, 2);
        assert_eq!(gateway.backend().select_calls, 0);
    }
}
