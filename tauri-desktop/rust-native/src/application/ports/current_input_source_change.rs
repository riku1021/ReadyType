use std::sync::Arc;

/// OS の入力ソース変更を受け取り、application 側でキャッシュを落とす。
pub type CurrentInputSourceChangeSink = Arc<dyn Fn() + Send + Sync>;

pub trait CurrentInputSourceChangeMonitor: Send {
    fn start(&mut self, sink: CurrentInputSourceChangeSink) -> Result<(), String>;
    fn stop(&mut self);
}
