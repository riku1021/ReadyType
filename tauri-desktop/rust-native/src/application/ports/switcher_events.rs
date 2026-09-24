use std::sync::Arc;

use crate::application::switcher_state::SwitcherState;

pub trait SwitcherEventPublisher: Send + Sync {
    fn publish(&self, state: &SwitcherState);
}

impl<T: SwitcherEventPublisher + ?Sized> SwitcherEventPublisher for Arc<T> {
    fn publish(&self, state: &SwitcherState) {
        (**self).publish(state);
    }
}
