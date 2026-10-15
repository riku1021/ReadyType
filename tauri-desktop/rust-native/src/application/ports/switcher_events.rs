use std::sync::Arc;

use crate::application::switcher_state::SwitcherState;

pub trait SwitcherEventPublisher: Send + Sync {
    /// Publish a cloned snapshot. Callers must invoke this **after** releasing
    /// the switcher mutex so tray / IPC side effects cannot re-enter the lock.
    fn publish(&self, state: &SwitcherState, locale: &str);

    /// Notify that the resolved UI pack changed while preference is `system`.
    fn publish_resolved_ui_locale(&self, locale: &str) {
        let _ = locale;
    }
}

impl<T: SwitcherEventPublisher + ?Sized> SwitcherEventPublisher for Arc<T> {
    fn publish(&self, state: &SwitcherState, locale: &str) {
        (**self).publish(state, locale);
    }

    fn publish_resolved_ui_locale(&self, locale: &str) {
        (**self).publish_resolved_ui_locale(locale);
    }
}
