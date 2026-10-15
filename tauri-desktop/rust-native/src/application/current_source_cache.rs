//! Short-lived cache for `gateway.current()` (PROTO_CURRENT_CACHE).

use std::time::{Duration, Instant};

use crate::domain::input_source::InputSourceId;

pub const DEFAULT_TTL: Duration = Duration::from_millis(2500);

#[derive(Debug, Default)]
pub struct CurrentInputSourceCache {
    entry: Option<(InputSourceId, Instant)>,
    ttl: Duration,
}

impl CurrentInputSourceCache {
    pub fn new() -> Self {
        Self {
            entry: None,
            ttl: DEFAULT_TTL,
        }
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self { entry: None, ttl }
    }

    pub fn get(&self) -> Option<&InputSourceId> {
        let (id, at) = self.entry.as_ref()?;
        if at.elapsed() <= self.ttl {
            Some(id)
        } else {
            None
        }
    }

    pub fn put(&mut self, id: InputSourceId) {
        self.entry = Some((id, Instant::now()));
    }

    pub fn invalidate(&mut self) {
        self.entry = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn hits_within_ttl() {
        let mut cache = CurrentInputSourceCache::with_ttl(Duration::from_millis(200));
        let id = InputSourceId::parse("en").expect("id");
        cache.put(id.clone());
        assert_eq!(cache.get().map(InputSourceId::as_str), Some("en"));
        thread::sleep(Duration::from_millis(250));
        assert!(cache.get().is_none());
    }

    #[test]
    fn invalidate_clears() {
        let mut cache = CurrentInputSourceCache::new();
        cache.put(InputSourceId::parse("jp").expect("id"));
        cache.invalidate();
        assert!(cache.get().is_none());
    }
}
