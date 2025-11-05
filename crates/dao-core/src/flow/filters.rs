//! Filter trait and implementations

use crate::Result;
use async_trait::async_trait;
use http::{Request, Response};

/// Trait для фильтра
#[async_trait]
pub trait Filter: Send + Sync {
    /// Обработка запроса
    async fn process_request<B>(&self, req: Request<B>) -> Result<Request<B>>
    where
        B: Send,
    {
        Ok(req)
    }

    /// Обработка ответа
    async fn process_response<B>(&self, res: Response<B>) -> Result<Response<B>>
    where
        B: Send,
    {
        Ok(res)
    }
}

/// Цепочка фильтров
pub struct FilterChain {
    // TODO: фильтры будут добавлены позже без dyn
}

impl FilterChain {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for FilterChain {
    fn default() -> Self {
        Self::new()
    }
}
