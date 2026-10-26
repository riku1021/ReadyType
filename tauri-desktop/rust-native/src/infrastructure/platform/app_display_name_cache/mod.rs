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
        if !self.inner.is_bundle_resolution_cacheable() {
            return self.inner.resolve(bundle_id);
        }
        self.resolve_cached(bundle_id, || self.inner.resolve(bundle_id))
    }

    fn resolve_for_process(&self, bundle_id: &str, process_id: i32) -> Option<String> {
        let cache_key = format!("{bundle_id}\0{process_id}");
        self.resolve_cached(&cache_key, || {
            self.inner.resolve_for_process(bundle_id, process_id)
        })
    }

    fn resolve_for_process_with_aumid(
        &self,
        bundle_id: &str,
        process_id: i32,
        app_user_model_id: Option<&str>,
    ) -> Option<String> {
        let cache_key = format!(
            "{bundle_id}\0{process_id}\0{}",
            app_user_model_id.unwrap_or_default()
        );
        self.resolve_cached(&cache_key, || {
            self.inner
                .resolve_for_process_with_aumid(bundle_id, process_id, app_user_model_id)
        })
    }
}

impl<R: AppDisplayNameResolver> CachedAppDisplayNameResolver<R> {
    fn resolve_cached(
        &self,
        bundle_id: &str,
        resolve: impl FnOnce() -> Option<String>,
    ) -> Option<String> {
        if bundle_id.is_empty() {
            return None;
        }

        if let Ok(memory) = self.memory.lock() {
            if let Some(cached) = memory.get(bundle_id) {
                return cached.clone();
            }
        }

        let resolved = resolve();
        if let Ok(mut memory) = self.memory.lock() {
            memory.insert(bundle_id.to_owned(), resolved.clone());
        }
        resolved
    }
}

#[cfg(test)]
mod tests;
