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
    last_locale: String,
}

impl FakeTis {
    fn with_lists(lists: Vec<Vec<NativeInputSource<FakeHandle>>>) -> Self {
        Self {
            lists: lists.into_iter().map(Ok).collect(),
            current: input_id("en"),
            list_calls: 0,
            select_calls: 0,
            selection_error: None,
            last_locale: String::new(),
        }
    }
}

impl TisBackend for FakeTis {
    type Handle = FakeHandle;

    fn list(&mut self, locale: &str) -> Result<Vec<NativeInputSource<Self::Handle>>, GatewayError> {
        self.list_calls += 1;
        self.last_locale = locale.to_owned();
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
    let mut gateway = MacOsInputSourceGateway::with_backend(backend, "ja").expect("initial list");

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
    let mut gateway = MacOsInputSourceGateway::with_backend(backend, "ja").expect("initial list");

    let available = gateway.list(false, "ja").expect("cached list");

    assert_eq!(available.len(), 1);
    assert_eq!(available[0].id.as_str(), "en");
    assert_eq!(available[0].name, "ABC");
    assert!(available[0].available);
}

#[test]
fn locale_change_refreshes_without_explicit_refresh_flag() {
    let backend = FakeTis::with_lists(vec![
        vec![source("en", "ABC", true, true, 1)],
        vec![source("en", "ABC", true, true, 1)],
    ]);
    let mut gateway = MacOsInputSourceGateway::with_backend(backend, "en").expect("initial list");
    assert_eq!(gateway.backend().list_calls, 1);

    gateway.list(false, "ja").expect("locale change");
    assert_eq!(gateway.backend().list_calls, 2);
    assert_eq!(gateway.backend().last_locale, "ja");
}

#[test]
fn selecting_current_source_still_calls_native_select() {
    let backend = FakeTis::with_lists(vec![vec![source("en", "ABC", true, true, 1)]]);
    let mut gateway = MacOsInputSourceGateway::with_backend(backend, "ja").expect("initial list");

    gateway.select(&input_id("en")).expect("already selected");

    assert_eq!(gateway.backend().select_calls, 1);
}

#[test]
fn native_selection_error_is_propagated() {
    let mut backend = FakeTis::with_lists(vec![vec![source("jp", "Japanese", true, true, 2)]]);
    backend.selection_error = Some(GatewayError::Operation("OSStatus -50".to_owned()));
    let mut gateway = MacOsInputSourceGateway::with_backend(backend, "ja").expect("initial list");

    let error = gateway
        .select(&input_id("jp"))
        .expect_err("selection fails");

    assert_eq!(
        error,
        GatewayError::Operation(
            "OSStatus -50; target=jp enabled=true selectCapable=true".to_owned()
        )
    );
    assert_eq!(gateway.backend().select_calls, 1);
}

#[test]
fn absent_target_after_refresh_fails_without_repeated_refreshes() {
    let backend = FakeTis::with_lists(vec![
        vec![source("en", "ABC", true, true, 1)],
        vec![source("en", "ABC", true, true, 1)],
    ]);
    let mut gateway = MacOsInputSourceGateway::with_backend(backend, "ja").expect("initial list");

    assert!(gateway.select(&input_id("jp")).is_err());
    assert_eq!(gateway.backend().list_calls, 2);
    assert_eq!(gateway.backend().select_calls, 0);
}
