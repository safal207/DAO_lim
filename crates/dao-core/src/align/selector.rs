//! Upstream selector trait

use crate::{Intent, upstream::UpstreamState};
use std::sync::Arc;

/// Trait для выбора upstream
pub trait UpstreamSelector: Send + Sync {
    /// Выбор upstream для запроса
    fn select(
        &self,
        upstreams: &[Arc<UpstreamState>],
        request_intent: Option<&Intent>,
    ) -> Option<Arc<UpstreamState>>;
}
