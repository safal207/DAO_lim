//! Rate limiting — token bucket per route
//!
//! Алгоритм token bucket:
//! - Вместимость = `rps` токенов (burst = 1 секунда)
//! - Пополнение: `rps` токенов в секунду непрерывно
//! - Каждый запрос потребляет 1 токен; если токенов нет — 429
//!
//! Использование:
//! ```ignore
//! let limiter = RateLimiterMap::new();
//! limiter.configure("api-v1", 1000);  // 1000 rps
//! if !limiter.check("api-v1") { /* return 429 */ }
//! ```

use dashmap::DashMap;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

/// Token bucket для одного маршрута
struct TokenBucket {
    tokens: Mutex<f64>,
    last_refill: Mutex<Instant>,
    /// Вместимость и скорость пополнения (токенов/сек)
    rate: f64,
}

impl TokenBucket {
    fn new(rps: u32) -> Self {
        let rate = rps as f64;
        Self {
            tokens: Mutex::new(rate), // начинаем с полным bucket
            last_refill: Mutex::new(Instant::now()),
            rate,
        }
    }

    /// Пытается потребить 1 токен.
    /// Возвращает `true` — запрос разрешён, `false` — превышен лимит (429).
    fn try_consume(&self) -> bool {
        let now = Instant::now();
        let mut tokens = self.tokens.lock();
        let mut last = self.last_refill.lock();

        let elapsed = now.duration_since(*last).as_secs_f64();
        *last = now;

        // Пополнение токенов пропорционально прошедшему времени
        *tokens = (*tokens + elapsed * self.rate).min(self.rate);

        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Хранилище rate limiter'ов по именам маршрутов.
/// Clone-able — всё внутри Arc<DashMap<...>>.
#[derive(Clone, Default)]
pub struct RateLimiterMap {
    buckets: Arc<DashMap<String, Arc<TokenBucket>>>,
}

impl RateLimiterMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Настройка лимита для маршрута (вызывается при старте из конфига).
    pub fn configure(&self, route_name: &str, rps: u32) {
        self.buckets
            .insert(route_name.to_string(), Arc::new(TokenBucket::new(rps)));
    }

    /// Проверка лимита. `true` = запрос разрешён, `false` = 429.
    /// Если лимит для маршрута не настроен — всегда `true`.
    pub fn check(&self, route_name: &str) -> bool {
        match self.buckets.get(route_name) {
            Some(bucket) => bucket.try_consume(),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiterMap::new();
        limiter.configure("test-route", 100);

        // Первые 100 запросов должны пройти (bucket заполнен)
        for _ in 0..100 {
            assert!(limiter.check("test-route"));
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiterMap::new();
        limiter.configure("test-route", 10);

        // Сначала дренируем bucket
        for _ in 0..10 {
            limiter.check("test-route");
        }
        // Следующий должен быть заблокирован
        assert!(!limiter.check("test-route"));
    }

    #[test]
    fn test_rate_limiter_no_config_always_allows() {
        let limiter = RateLimiterMap::new();
        for _ in 0..1000 {
            assert!(limiter.check("unconfigured-route"));
        }
    }

    #[test]
    fn test_rate_limiter_refills_over_time() {
        let limiter = RateLimiterMap::new();
        limiter.configure("test-route", 10);

        // Дренируем
        for _ in 0..10 {
            limiter.check("test-route");
        }
        assert!(!limiter.check("test-route"));

        // Эмулируем прошедшее время через прямой доступ к bucket
        {
            let bucket = limiter.buckets.get("test-route").unwrap();
            let mut last = bucket.last_refill.lock();
            *last = Instant::now() - std::time::Duration::from_secs(1);
        }

        // После 1 секунды должно быть снова доступно
        assert!(limiter.check("test-route"));
    }

    #[test]
    fn test_rate_limiter_clone_shares_state() {
        let limiter = RateLimiterMap::new();
        limiter.configure("route", 5);
        let clone = limiter.clone();

        // Дренируем через оригинал
        for _ in 0..5 {
            limiter.check("route");
        }
        // Клон видит тот же bucket
        assert!(!clone.check("route"));
    }
}
