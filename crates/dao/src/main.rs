//! DAO — Dynamic Awareness Orchestrator
//!
//! Лиминальный reverse-proxy с осознанной маршрутизацией

use clap::Parser;
use dao_admin::Admin;
use dao_core::{
    align::Align,
    config::DaoConfig,
    gate::{Gate, GateConfig, TlsConfig},
    memory::Memory,
    sense::Sense,
    upstream::UpstreamState,
};
use dao_telemetry::{init_telemetry, register_dao_metrics, start_prometheus_exporter};
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

    // Инициализация телеметрии
    init_telemetry()?;
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
            let upstream = UpstreamState::new(
                upstream_cfg.name.clone(),
                upstream_cfg.url.clone(),
                upstream_cfg.intents(),
                upstream_cfg.weight,
            );
            all_upstreams.push(upstream);
        }
    }
    let upstreams = Arc::new(all_upstreams);

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

    // Создание и запуск сервера
    let server = DaoServer::new(gate, sense, align, memory, upstreams);

    info!("DAO started successfully");
    info!("Dynamic Awareness Orchestrator — врата сознания открыты");

    server.run().await?;

    Ok(())
}
