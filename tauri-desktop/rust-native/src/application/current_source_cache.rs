use std::time::{Duration, Instant};

use crate::domain::input_source::InputSourceId;

/// `gateway.current()` の短時間キャッシュ。TIS の間欠スパイクを抑える。
#[derive(Debug)]
pub struct CurrentInputSourceCache {
    entry: Option<(Instant, InputSourceId)>,
    ttl: Duration,
}

impl CurrentInputSourceCache {
    /// フォーカス間隔が数百 ms を超えてもヒットしやすいよう、短すぎない TTL にする。
    /// 手動 IME 切替は次フォーカス or 明示 invalidate で拾う（OS 通知は無い）。
    pub const DEFAULT_TTL: Duration = Duration::from_millis(2_500);

    pub fn new(ttl: Duration) -> Self {
        Self { entry: None, ttl }
    }

    pub fn get(&self, now: Instant) -> Option<InputSourceId> {
        let (stored_at, id) = self.entry.as_ref()?;
        if now.duration_since(*stored_at) <= self.ttl {
            Some(id.clone())
        } else {
            None
        }
    }

    pub fn put(&mut self, now: Instant, id: InputSourceId) {
        self.entry = Some((now, id));
    }

    pub fn invalidate(&mut self) {
        self.entry = None;
    }
}

impl Default for CurrentInputSourceCache {
    fn default() -> Self {
        Self::new(Self::DEFAULT_TTL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_cached_value_within_ttl() {
        let mut cache = CurrentInputSourceCache::new(Duration::from_millis(100));
        let id = InputSourceId::parse("en").expect("id");
        let now = Instant::now();
        cache.put(now, id.clone());
        assert_eq!(cache.get(now), Some(id));
    }

    #[test]
    fn expires_after_ttl() {
        let mut cache = CurrentInputSourceCache::new(Duration::from_millis(1));
        let id = InputSourceId::parse("en").expect("id");
        let now = Instant::now();
        cache.put(now, id);
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(cache.get(Instant::now()), None);
    }

    #[test]
    fn default_ttl_is_long_enough_for_focus_gaps() {
        assert!(CurrentInputSourceCache::DEFAULT_TTL >= Duration::from_millis(2_000));
    }
}
