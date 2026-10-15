//! Process-lifetime memory cache for [`AppDisplayNameResolver`].

use std::collections::HashMap;
use std::sync::Mutex;

use crate::application::ports::app_display_name::AppDisplayNameResolver;

/// Caches display-name resolutions for the process lifetime.
pub struct CachedAppDisplayNameResolver<R: AppDisplayNameResolver> {
    inner: R,
    memory: Mutex<HashMap<String, Option<String>>>,
}

impl<R: AppDisplayNameResolver> CachedAppDisplayNameResolver<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            memory: Mutex::new(HashMap::new()),
        }
    }
}

impl<R: AppDisplayNameResolver> AppDisplayNameResolver for CachedAppDisplayNameResolver<R> {
    fn resolve(&self, bundle_id: &str) -> Option<String> {
        if bundle_id.is_empty() {
            return None;
        }

        if let Ok(memory) = self.memory.lock() {
            if let Some(cached) = memory.get(bundle_id) {
                return cached.clone();
            }
        }

        let resolved = self.inner.resolve(bundle_id);
        if let Ok(mut memory) = self.memory.lock() {
            memory.insert(bundle_id.to_owned(), resolved.clone());
        }
        resolved
    }
}

#[cfg(test)]
mod tests;
