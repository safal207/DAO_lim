//! Gate — врата сознания
//!
//! Модуль приема входящих соединений:
//! - TCP/TLS listeners
//! - ALPN negotiation (h1/h2)
//! - SNI routing (будущее)

use crate::Result;
use rustls::ServerConfig;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

pub mod listener;

pub use listener::{GateListener, Connection, Protocol};

/// Конфигурация Gate
#[derive(Debug, Clone)]
pub struct GateConfig {
    pub bind_addr: String,
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

/// Gate — точка входа в систему
pub struct Gate {
    listener: TcpListener,
    tls_acceptor: Option<TlsAcceptor>,
}

impl Gate {
    /// Создание нового Gate
    pub async fn new(config: GateConfig) -> Result<Self> {
        let listener = TcpListener::bind(&config.bind_addr).await?;

        let tls_acceptor = if let Some(tls_cfg) = config.tls {
            let tls_acceptor = create_tls_acceptor(&tls_cfg).await?;
            Some(tls_acceptor)
        } else {
            None
        };

        Ok(Self {
            listener,
            tls_acceptor,
        })
    }

    /// Получение следующего соединения
    pub async fn accept(&self) -> Result<Connection> {
        let (stream, peer_addr) = self.listener.accept().await?;

        let connection = if let Some(acceptor) = &self.tls_acceptor {
            // TLS handshake
            let tls_stream = acceptor.accept(stream).await
                .map_err(|e| crate::DaoError::Tls(e.to_string()))?;

            // Определение протокола через ALPN
            let protocol = detect_alpn_protocol(&tls_stream);

            Connection::Tls {
                stream: Box::new(tls_stream),
                peer_addr,
                protocol,
            }
        } else {
            Connection::Plain {
                stream,
                peer_addr,
                protocol: Protocol::Http1,
            }
        };

        Ok(connection)
    }

    /// Получение локального адреса
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }
}

/// Создание TLS acceptor из конфигурации
async fn create_tls_acceptor(config: &TlsConfig) -> Result<TlsAcceptor> {
    use rustls_pemfile::{certs, private_key};

    let cert_file = std::fs::File::open(&config.cert_path)?;
    let key_file = std::fs::File::open(&config.key_path)?;

    let mut cert_reader = std::io::BufReader::new(cert_file);
    let mut key_reader = std::io::BufReader::new(key_file);

    let cert_chain: Vec<_> = certs(&mut cert_reader)
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|e| crate::DaoError::Tls(format!("Failed to read certs: {}", e)))?;

    let key = private_key(&mut key_reader)
        .map_err(|e| crate::DaoError::Tls(format!("Failed to read key: {}", e)))?
        .ok_or_else(|| crate::DaoError::Tls("No private key found".to_string()))?;

    let mut tls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .map_err(|e| crate::DaoError::Tls(e.to_string()))?;

    // ALPN protocols: h2, http/1.1
    tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(TlsAcceptor::from(Arc::new(tls_config)))
}

/// Определение протокола из ALPN
fn detect_alpn_protocol<S>(stream: &tokio_rustls::server::TlsStream<S>) -> Protocol {
    let (_, session) = stream.get_ref();

    if let Some(alpn) = session.alpn_protocol() {
        match alpn {
            b"h2" => Protocol::Http2,
            b"http/1.1" => Protocol::Http1,
            _ => Protocol::Http1,
        }
    } else {
        Protocol::Http1
    }
}
