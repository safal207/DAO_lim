//! DAO — Dynamic Awareness Orchestrator
//!
//! Лиминальный reverse-proxy с осознанной маршрутизацией

use clap::Parser;
use dao_admin::{Admin, AdminState, start_admin_api};
use dao_core::{
    align::Align,
    config::DaoConfig,
    gate::{Gate, GateConfig, TlsConfig},
    memory::Memory,
    sense::Sense,
    upstream::{UpstreamState, HealthCheckConfig, spawn_health_checker},
};
use dao_telemetry::{init_telemetry_with_otlp, register_dao_metrics, start_prometheus_exporter};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

mod server;
use server::DaoServer;

#[derive(Parser, Debug)]
#[command(name = "dao")]
#[command(about = "Dynamic Awareness Orchestrator — лиминальный reverse-proxy", long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "configs/dao.toml")]
    config: PathBuf,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Загрузка конфигурации нужна до init_telemetry для otlp_endpoint,
    // но clap Args уже распарсены, поэтому читаем конфиг дважды
    // (первый раз только для endpoint — это дёшево).
    let otlp_endpoint = DaoConfig::from_file(&args.config)
        .ok()
        .and_then(|c| c.telemetry)
        .and_then(|t| t.otlp_endpoint);

    init_telemetry_with_otlp(otlp_endpoint.as_deref())?;
    register_dao_metrics();

    if args.verbose {
        info!("Verbose logging enabled");
    }

    // Загрузка конфигурации
    info!("Loading configuration from: {:?}", args.config);
    let config = DaoConfig::from_file(&args.config)?;
    config.validate()?;

    info!("Configuration loaded successfully");

    // Создание компонентов DAO
    let memory = Arc::new(Memory::new(config.clone()));

    // Создание upstream states
    let mut all_upstreams = Vec::new();
    for route in &config.routes.rule {
        for upstream_cfg in &route.upstreams {
            use dao_core::upstream::CircuitBreakerConfig as CbConfig;
            let cb_config = upstream_cfg.circuit_breaker.as_ref().map(|c| CbConfig {
                failure_threshold: c.failure_threshold,
                success_threshold: c.success_threshold,
                timeout: std::time::Duration::from_secs(c.timeout_secs),
            }).unwrap_or_default();
            let upstream = UpstreamState::with_circuit(
                upstream_cfg.name.clone(),
                upstream_cfg.url.clone(),
                upstream_cfg.intents(),
                upstream_cfg.weight,
                cb_config,
            );
            all_upstreams.push(upstream);
        }
    }
    let upstreams = Arc::new(all_upstreams);

    // Запуск активных health checkers для upstream'ов с настроенным health_check
    for route in &config.routes.rule {
        for upstream_cfg in &route.upstreams {
            if let Some(ref hc) = upstream_cfg.health_check {
                if let Some(upstream) = upstreams.iter().find(|u| u.name == upstream_cfg.name) {
                    let hc_config = HealthCheckConfig {
                        path: hc.path.clone(),
                        interval_secs: hc.interval_secs,
                        timeout_secs: hc.timeout_secs,
                    };
                    info!("Starting health checker for '{}' at {} every {}s",
                        upstream.name, hc.path, hc.interval_secs);
                    spawn_health_checker(upstream.clone(), hc_config);
                }
            }
        }
    }

    // Sense — телеметрия
    let sense = Sense::new(upstreams.clone());

    // Align — политики
    let mut align = Align::new(sense.clone());

    // Регистрация политик из конфига
    if let Some(policies) = &config.policies {
        for (name, policy_cfg) in policies {
            use dao_core::align::PolicyWeights;
            let weights = PolicyWeights::new(
                policy_cfg.w_load,
                policy_cfg.w_intent,
                policy_cfg.w_tempo,
            );
            align.register_policy(name.clone(), weights);
        }
    }

    // Admin — управление
    let admin = Arc::new(Admin::new(args.config.clone(), memory.clone()));

    // Gate — прием соединений
    let gate_config = GateConfig {
        bind_addr: config.server.bind.clone(),
        tls: config
            .server
            .tls_cert
            .as_ref()
            .zip(config.server.tls_key.as_ref())
            .map(|(cert, key)| TlsConfig {
                cert_path: cert.clone(),
                key_path: key.clone(),
            }),
    };

    let gate = Gate::new(gate_config).await?;
    let local_addr = gate.local_addr()?;
    info!("DAO listening on: {}", local_addr);

    // Запуск Prometheus exporter
    if let Some(telemetry_cfg) = &config.telemetry {
        let prometheus_addr = telemetry_cfg.prometheus_bind.parse()?;
        tokio::spawn(async move {
            if let Err(e) = start_prometheus_exporter(prometheus_addr).await {
                error!("Failed to start Prometheus exporter: {}", e);
            }
        });
    }

    // Запуск config watch
    tokio::spawn({
        let admin = admin.clone();
        async move {
            if let Err(e) = admin.start_config_watch().await {
                error!("Config watch failed: {}", e);
            }
        }
    });

    // Запуск Admin API (daoctl)
    if let Some(ref admin_bind) = config.server.admin_bind {
        let admin_addr: std::net::SocketAddr = admin_bind.parse()?;
        let admin_state = AdminState {
            upstreams: upstreams.clone(),
            memory: memory.clone(),
            sense: Arc::new(sense.clone()),
            align: Arc::new(align.clone()),
        };
        tokio::spawn(async move {
            if let Err(e) = start_admin_api(admin_addr, admin_state).await {
                error!("Admin API failed: {}", e);
            }
        });
    }

    // Создание и запуск сервера
    let server = DaoServer::new(gate, sense, align, memory, upstreams);

    info!("DAO started successfully");
    info!("Dynamic Awareness Orchestrator — врата сознания открыты");

    server.run().await?;

    Ok(())
}
