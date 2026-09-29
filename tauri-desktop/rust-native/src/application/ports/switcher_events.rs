use std::sync::Arc;

use crate::application::switcher_state::SwitcherState;

pub trait SwitcherEventPublisher: Send + Sync {
    /// Publish a cloned snapshot. Callers must invoke this **after** releasing
    /// the switcher mutex so tray / IPC side effects cannot re-enter the lock.
    fn publish(&self, state: &SwitcherState, locale: &str);
}

impl<T: SwitcherEventPublisher + ?Sized> SwitcherEventPublisher for Arc<T> {
    fn publish(&self, state: &SwitcherState, locale: &str) {
        (**self).publish(state, locale);
    }
}
