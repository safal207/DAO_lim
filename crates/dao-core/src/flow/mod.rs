//! Flow — поток и трансформации
//!
//! Модуль фильтров и обработки запросов:
//! - Header manipulation
//! - Rate limiting
//! - Authentication
//! - Compression
//! - WASM filters (будущее)

use crate::Result;
use http::{Request, Response, HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;

pub mod filters;
pub use filters::{Filter, FilterChain};

/// Flow — система обработки потока
pub struct Flow {
    // TODO: фильтры будут добавлены позже с конкретными типами
}

impl Flow {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Flow {
    fn default() -> Self {
        Self::new()
    }
}

/// Header manipulation utilities
pub struct HeaderManipulator {
    add_headers: HashMap<String, String>,
    remove_headers: Vec<String>,
}

impl HeaderManipulator {
    pub fn new() -> Self {
        Self {
            add_headers: HashMap::new(),
            remove_headers: Vec::new(),
        }
    }

    pub fn add_header(&mut self, key: String, value: String) {
        self.add_headers.insert(key, value);
    }

    pub fn remove_header(&mut self, key: String) {
        self.remove_headers.push(key);
    }

    pub fn apply_to_headers(&self, headers: &mut HeaderMap) -> Result<()> {
        // Добавление заголовков
        for (key, value) in &self.add_headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| crate::DaoError::Filter(format!("Invalid header name: {}", e)))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|e| crate::DaoError::Filter(format!("Invalid header value: {}", e)))?;
            headers.insert(header_name, header_value);
        }

        // Удаление заголовков
        for key in &self.remove_headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| crate::DaoError::Filter(format!("Invalid header name: {}", e)))?;
            headers.remove(header_name);
        }

        Ok(())
    }
}

impl Default for HeaderManipulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_manipulator() {
        let mut manipulator = HeaderManipulator::new();
        manipulator.add_header("x-dao".to_string(), "true".to_string());
        manipulator.remove_header("x-unwanted".to_string());

        let mut headers = HeaderMap::new();
        headers.insert("x-unwanted", "value".parse().unwrap());

        manipulator.apply_to_headers(&mut headers).unwrap();

        assert!(headers.contains_key("x-dao"));
        assert!(!headers.contains_key("x-unwanted"));
    }
}
