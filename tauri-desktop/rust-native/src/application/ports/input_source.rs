use std::fmt;

use crate::domain::input_source::InputSourceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableInputSource {
    pub id: InputSourceId,
    pub name: String,
    /// BCP 47 language code（家族推論で正規化済みの場合あり）。無い場合は `None`。
    pub language_code: Option<String>,
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayError {
    UnsupportedPlatform,
    Operation(String),
}

impl fmt::Display for GatewayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(formatter, "input sources are unsupported"),
            Self::Operation(message) => {
                write!(formatter, "input source operation failed: {message}")
            }
        }
    }
}

impl std::error::Error for GatewayError {}

pub trait InputSourceGateway: Send {
    /// `locale` は UI 表示言語（`ja` / `en` / `zh`）。表示名解決に使う。
    fn list(
        &mut self,
        refresh: bool,
        locale: &str,
    ) -> Result<Vec<AvailableInputSource>, GatewayError>;
    fn current(&self) -> Result<InputSourceId, GatewayError>;
    fn select(&mut self, id: &InputSourceId) -> Result<(), GatewayError>;
    /// 現在の IME に紐づくキーボードレイアウト override（無ければ `None`）。
    fn input_method_keyboard_layout_override(&self) -> Option<InputSourceId> {
        None
    }
}

impl<T: InputSourceGateway + ?Sized> InputSourceGateway for Box<T> {
    fn list(
        &mut self,
        refresh: bool,
        locale: &str,
    ) -> Result<Vec<AvailableInputSource>, GatewayError> {
        (**self).list(refresh, locale)
    }

    fn current(&self) -> Result<InputSourceId, GatewayError> {
        (**self).current()
    }

    fn select(&mut self, id: &InputSourceId) -> Result<(), GatewayError> {
        (**self).select(id)
    }

    fn input_method_keyboard_layout_override(&self) -> Option<InputSourceId> {
        (**self).input_method_keyboard_layout_override()
    }
}
