//! DAO Server — обработка запросов

use dao_core::{
    align::Align,
    gate::{Connection, Gate, Protocol},
    memory::Memory,
    sense::Sense,
    upstream::UpstreamState,
    Intent, Result,
};
use http_body_util::{BodyExt, Empty};
use hyper::{body::Bytes, body::Incoming, service::service_fn, Request, Response};
use std::sync::Arc;
use tracing::{debug, error, info};

/// DAO Server
pub struct DaoServer {
    gate: Gate,
    sense: Sense,
    align: Align,
    memory: Arc<Memory>,
    upstreams: Arc<Vec<UpstreamState>>,
}

impl DaoServer {
    pub fn new(
        gate: Gate,
        sense: Sense,
        align: Align,
        memory: Arc<Memory>,
        upstreams: Arc<Vec<UpstreamState>>,
    ) -> Self {
        Self {
            gate,
            sense,
            align,
            memory,
            upstreams,
        }
    }

    /// Запуск сервера
    pub async fn run(self) -> anyhow::Result<()> {
        let self_arc = Arc::new(self);

        loop {
            match self_arc.gate.accept().await {
                Ok(conn) => {
                    let server = self_arc.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(conn).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }

    /// Обработка соединения
    async fn handle_connection(&self, conn: Connection) -> Result<()> {
        let peer_addr = conn.peer_addr();
        let protocol = conn.protocol();

        debug!(
            "New connection from {} with protocol {:?}",
            peer_addr, protocol
        );

        match protocol {
            Protocol::Http1 | Protocol::Http2 => {
                self.handle_http_connection(conn).await?;
            }
            Protocol::WebSocket => {
                self.handle_websocket_connection(conn).await?;
            }
        }

        Ok(())
    }

    /// Обработка HTTP соединения
    async fn handle_http_connection(&self, conn: Connection) -> Result<()> {
        // Simplified HTTP handler
        // В полной версии здесь будет hyper server
        info!("HTTP connection handled (placeholder)");
        Ok(())
    }

    /// Обработка WebSocket соединения
    async fn handle_websocket_connection(&self, conn: Connection) -> Result<()> {
        info!("WebSocket connection handled (placeholder)");
        Ok(())
    }

    /// Обработка HTTP запроса
    async fn handle_request(
        self: Arc<Self>,
        req: Request<Incoming>,
    ) -> Result<Response<Empty<Bytes>>> {
        let config = self.memory.get_config();

        // Поиск подходящего маршрута
        let route = config
            .routes
            .rule
            .iter()
            .find(|r| r.match_rule.matches(&req));

        if let Some(route) = route {
            debug!("Matched route: {}", route.name);

            // Получение upstream'ов для маршрута
            let route_upstreams: Vec<_> = route
                .upstreams
                .iter()
                .filter_map(|uc| {
                    self.upstreams
                        .iter()
                        .find(|u| u.name == uc.name)
                        .map(|u| Arc::new(u.clone()))
                })
                .collect();

            if route_upstreams.is_empty() {
                return Ok(Response::builder()
                    .status(503)
                    .body(Empty::new())
                    .unwrap());
            }

            // Выбор upstream через Align
            let request_intent = route.intent();
            let selected = self
                .align
                .select_upstream(&route.policy, &route_upstreams, request_intent.as_ref());

            if let Some(upstream) = selected {
                info!(
                    "Selected upstream: {} for route: {}",
                    upstream.name, route.name
                );

                // TODO: Проксирование запроса к upstream
                // Пока возвращаем 200 OK

                Ok(Response::builder().status(200).body(Empty::new()).unwrap())
            } else {
                Ok(Response::builder()
                    .status(503)
                    .body(Empty::new())
                    .unwrap())
            }
        } else {
            // Маршрут не найден
            Ok(Response::builder()
                .status(404)
                .body(Empty::new())
                .unwrap())
        }
    }
}
