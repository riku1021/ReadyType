use std::sync::Arc;

pub trait Clock: Send + Sync {
    fn now_unix_ms(&self) -> u64;
}

impl<T: Clock + ?Sized> Clock for Arc<T> {
    fn now_unix_ms(&self) -> u64 {
        (**self).now_unix_ms()
    }
}
