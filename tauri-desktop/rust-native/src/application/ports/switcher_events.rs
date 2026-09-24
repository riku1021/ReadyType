use crate::application::switcher_state::SwitcherState;

pub trait SwitcherEventPublisher: Send + Sync {
    fn publish(&self, state: &SwitcherState);
}
