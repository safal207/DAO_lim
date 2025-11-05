//! Listener types and connection handling

use std::net::SocketAddr;
use tokio::net::TcpStream;

/// Протокол соединения
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Http1,
    Http2,
    WebSocket,
}

/// Входящее соединение
pub enum Connection {
    /// Plain TCP (HTTP)
    Plain {
        stream: TcpStream,
        peer_addr: SocketAddr,
        protocol: Protocol,
    },
    /// TLS
    Tls {
        stream: Box<tokio_rustls::server::TlsStream<TcpStream>>,
        peer_addr: SocketAddr,
        protocol: Protocol,
    },
}

impl Connection {
    /// Получение peer address
    pub fn peer_addr(&self) -> SocketAddr {
        match self {
            Connection::Plain { peer_addr, .. } => *peer_addr,
            Connection::Tls { peer_addr, .. } => *peer_addr,
        }
    }

    /// Получение протокола
    pub fn protocol(&self) -> Protocol {
        match self {
            Connection::Plain { protocol, .. } => *protocol,
            Connection::Tls { protocol, .. } => *protocol,
        }
    }
}

/// Listener abstraction
pub trait GateListener: Send + Sync {
    fn accept(&self) -> impl std::future::Future<Output = std::io::Result<Connection>> + Send;
}
