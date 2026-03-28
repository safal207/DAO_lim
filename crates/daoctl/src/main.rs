//! daoctl — CLI для управления и отладки DAO
//!
//! Команды:
//!   daoctl explain --host api.example.com --path /v1/users --intent realtime
//!   daoctl upstreams
//!   daoctl health

use anyhow::Result;
use clap::{Parser, Subcommand};
use dao_core::explain::{CandidateScore, CanaryResponse, ExplainResult, UpstreamsResponse};

#[derive(Parser, Debug)]
#[command(name = "daoctl")]
#[command(about = "DAO CLI — мониторинг и отладка маршрутизации", long_about = None)]
#[command(version)]
struct Cli {
    /// Адрес admin API (по умолчанию: http://127.0.0.1:9103)
    #[arg(short, long, default_value = "http://127.0.0.1:9103", global = true)]
    server: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Объяснить решение маршрутизации для заданного запроса
    Explain {
        /// Host заголовок
        #[arg(long)]
        host: Option<String>,

        /// Путь запроса (например /v1/users)
        #[arg(long)]
        path: Option<String>,

        /// HTTP метод
        #[arg(long, default_value = "GET")]
        method: String,

        /// Intent тег (realtime, batch, streaming, ai)
        #[arg(long)]
        intent: Option<String>,

        /// Вывести сырой JSON
        #[arg(long)]
        json: bool,
    },

    /// Показать состояние всех upstream
    Upstreams {
        /// Вывести сырой JSON
        #[arg(long)]
        json: bool,
    },

    /// Показать canary-раскладку трафика
    Canary {
        /// Вывести сырой JSON
        #[arg(long)]
        json: bool,
    },

    /// Проверить доступность DAO
    Health,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Explain { host, path, method, intent, json } => {
            cmd_explain(&cli.server, host, path, method, intent, json).await?;
        }
        Command::Upstreams { json } => {
            cmd_upstreams(&cli.server, json).await?;
        }
        Command::Canary { json } => {
            cmd_canary(&cli.server, json).await?;
        }
        Command::Health => {
            cmd_health(&cli.server).await?;
        }
    }

    Ok(())
}

// ── explain ──────────────────────────────────────────────────────────────────

async fn cmd_explain(
    server: &str,
    host: Option<String>,
    path: Option<String>,
    method: String,
    intent: Option<String>,
    raw_json: bool,
) -> Result<()> {
    let mut params = vec![format!("method={}", method)];
    if let Some(ref h) = host { params.push(format!("host={}", h)); }
    if let Some(ref p) = path { params.push(format!("path={}", urlenc(p))); }
    if let Some(ref i) = intent { params.push(format!("intent={}", i)); }

    let url = format!("{}/admin/explain?{}", server, params.join("&"));
    let body = get_json(&url).await?;

    if raw_json {
        println!("{}", body);
        return Ok(());
    }

    let result: ExplainResult = serde_json::from_str(&body)
        .map_err(|e| anyhow::anyhow!("Failed to parse response: {}\nBody: {}", e, body))?;

    print_explain(&result, host.as_deref(), path.as_deref(), &method, intent.as_deref());
    Ok(())
}

fn print_explain(
    r: &ExplainResult,
    host: Option<&str>,
    path: Option<&str>,
    method: &str,
    intent: Option<&str>,
) {
    println!();
    println!("  {} {} {}", bold("REQUEST"), method, path.unwrap_or("/"));
    if let Some(h) = host { println!("  Host:   {}", h); }
    if let Some(i) = intent { println!("  Intent: {}", i); }
    println!();

    if r.no_route {
        println!("  {} Маршрут не найден.", red("✗"));
        println!("  Проверьте host/path в configs/dao.toml");
        println!();
        return;
    }

    println!("  {} Route:  {}", dim("→"), r.route);
    println!("  {} Policy: {}", dim("→"), r.policy);
    println!(
        "  {} Веса:   w_load={:.2}  w_intent={:.2}  w_tempo={:.2}",
        dim("→"), r.weights.w_load, r.weights.w_intent, r.weights.w_tempo
    );
    println!();

    if let Some(ref winner) = r.selected {
        println!("  {} Выбран: {}", green("✓"), bold(winner));
    } else {
        println!("  {} Нет подходящего upstream", red("✗"));
    }
    println!();

    // Таблица кандидатов
    println!("  {:<22} {:>8}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}",
        bold("UPSTREAM"), "SCORE", "p95ms", "err%", "RPS",
        "load×w", "intent×w",
    );
    println!("  {}", "─".repeat(84));

    for c in &r.candidates {
        let marker = if c.winner { green("▶") } else { dim("·") };
        println!(
            "  {} {:<20} {:>8.4}  {:>8.1}  {:>7.1}%  {:>8.1}  {:>8.4}  {:>8.4}",
            marker,
            c.name,
            c.total_score,
            c.p95_latency_ms,
            c.error_rate * 100.0,
            c.current_rps,
            c.components.load_component,
            c.components.intent_component,
        );

        if c.winner {
            print_why_winner(c, &r.weights.w_load, &r.weights.w_intent, &r.weights.w_tempo);
        }
    }
    println!();
}

fn print_why_winner(c: &CandidateScore, w_load: &f64, w_intent: &f64, w_tempo: &f64) {
    println!();
    println!("    {} Почему выбран {}:", dim("┌"), c.name);
    println!("    {}   load_resonance  = {:.4}  (p95={:.1}ms, err={:.1}%)",
        dim("│"), c.load_resonance, c.p95_latency_ms, c.error_rate * 100.0);
    println!("    {}   intent_gap      = {:.4}  ({})",
        dim("│"), c.intent_gap,
        if c.intent_gap == 0.0 { "intent совпадает" } else { "intent не совпадает" });
    println!("    {}   tempo_spikiness = {:.4}",
        dim("│"), c.tempo_spikiness);
    println!("    {}   score = {:.2}×{:.4} + {:.2}×{:.4} + {:.2}×{:.4} = {:.4}",
        dim("└"),
        w_load, c.load_resonance,
        w_intent, c.intent_gap,
        w_tempo, c.tempo_spikiness,
        c.total_score,
    );
    println!();
}

// ── upstreams ─────────────────────────────────────────────────────────────────

async fn cmd_upstreams(server: &str, raw_json: bool) -> Result<()> {
    let url = format!("{}/admin/upstreams", server);
    let body = get_json(&url).await?;

    if raw_json {
        println!("{}", body);
        return Ok(());
    }

    let resp: UpstreamsResponse = serde_json::from_str(&body)
        .map_err(|e| anyhow::anyhow!("Failed to parse: {}\nBody: {}", e, body))?;

    println!();
    println!("  {:<22} {:>8}  {:>8}  {:>7}  {:>8}  {:>10}  {:>10}",
        bold("UPSTREAM"), "p50ms", "p95ms", "err%", "RPS", "ok", "fail",
    );
    println!("  {}", "─".repeat(88));

    for u in &resp.upstreams {
        let err_pct = u.error_rate * 100.0;
        let health = if err_pct > 10.0 {
            red("✗")
        } else if err_pct > 2.0 {
            yellow("!")
        } else {
            green("✓")
        };

        println!(
            "  {} {:<20} {:>8.1}  {:>8.1}  {:>6.1}%  {:>8.1}  {:>10}  {:>10}",
            health,
            u.name,
            u.p50_latency_ms,
            u.p95_latency_ms,
            err_pct,
            u.current_rps,
            u.success_count,
            u.error_count,
        );
        println!("    {} {} | load_resonance={:.4}  tempo_spikiness={:.4}",
            dim("└"), dim(&u.url), u.load_resonance, u.tempo_spikiness);
    }
    println!();

    Ok(())
}

// ── canary ────────────────────────────────────────────────────────────────────

async fn cmd_canary(server: &str, raw_json: bool) -> Result<()> {
    let url = format!("{}/admin/canary", server);
    let body = get_json(&url).await?;

    if raw_json {
        println!("{}", body);
        return Ok(());
    }

    let resp: CanaryResponse = serde_json::from_str(&body)
        .map_err(|e| anyhow::anyhow!("Failed to parse: {}\nBody: {}", e, body))?;

    if resp.routes.is_empty() {
        println!("\n  {} Нет маршрутов с policy=\"canary\"\n", dim("·"));
        println!("  Добавьте в dao.toml:");
        println!("    {}", dim("[[routes.rule]]"));
        println!("    {}", dim("policy = \"canary\""));
        println!();
        return Ok(());
    }

    for route in &resp.routes {
        println!();
        println!("  {} Route: {}  (всего запросов: {})",
            bold("◆"), bold(&route.route), route.total_requests);
        println!();
        println!("  {:<22} {:>8}  {:>8}  {:>8}  {:>8}  {:>8}  {}",
            bold("UPSTREAM"), "weight%", "actual%", "p95ms", "err%", "reqs", "circuit");
        println!("  {}", "─".repeat(82));

        for u in &route.upstreams {
            let circuit_label = match u.circuit.label() {
                "closed"    => green("closed"),
                "open"      => red("open ⚡"),
                "half_open" => yellow("half_open"),
                other       => dim(other),
            };

            let actual_marker = if (u.requests_pct - u.weight_pct).abs() > 5.0 {
                yellow("!")
            } else {
                dim(" ")
            };

            println!(
                "  {} {:<20} {:>7.1}%  {:>7.1}%  {:>8.1}  {:>7.1}%  {:>8}  {}",
                actual_marker,
                u.name,
                u.weight_pct,
                u.requests_pct,
                u.p95_latency_ms,
                u.error_rate * 100.0,
                u.requests_total,
                circuit_label,
            );
        }

        println!();
        println!("  {} weight% = конфигурированная доля трафика", dim("·"));
        println!("  {} actual% = фактическая доля по счётчикам", dim("·"));
        println!("  {} {} — отклонение actual% от weight% > 5pp", yellow("!"), dim("маркер"));
    }
    println!();

    Ok(())
}

// ── health ────────────────────────────────────────────────────────────────────

async fn cmd_health(server: &str) -> Result<()> {
    let url = format!("{}/admin/health", server);
    let body = get_json(&url).await?;
    let val: serde_json::Value = serde_json::from_str(&body)?;
    let status = val["status"].as_str().unwrap_or("unknown");
    let version = val["version"].as_str().unwrap_or("?");

    if status == "ok" {
        println!("{} DAO is running  (v{})", green("✓"), version);
    } else {
        println!("{} DAO status: {}", red("✗"), status);
    }
    Ok(())
}

// ── HTTP client ───────────────────────────────────────────────────────────────

async fn get_json(url: &str) -> Result<String> {
    use http_body_util::BodyExt;
    use hyper::Request;
    use hyper_util::rt::TokioIo;
    use tokio::net::TcpStream;

    let uri: http::Uri = url.parse()?;
    let host = uri.host().ok_or_else(|| anyhow::anyhow!("No host in URL"))?;
    let port = uri.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);

    let stream = TcpStream::connect(&addr).await
        .map_err(|e| anyhow::anyhow!("Cannot connect to {}: {}\nIs DAO running? Check admin_bind in dao.toml", addr, e))?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::spawn(async move { let _ = conn.await; });

    let path_and_query = uri.path_and_query().map(|p| p.as_str()).unwrap_or("/");
    let req = Request::builder()
        .method("GET")
        .uri(path_and_query)
        .header("Host", host)
        .body(http_body_util::Empty::<hyper::body::Bytes>::new())?;

    let resp = sender.send_request(req).await?;
    let bytes = resp.into_body().collect().await?.to_bytes();
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

// ── ANSI colour helpers ───────────────────────────────────────────────────────

fn green(s: &str) -> String  { format!("\x1b[32m{}\x1b[0m", s) }
fn red(s: &str) -> String    { format!("\x1b[31m{}\x1b[0m", s) }
fn yellow(s: &str) -> String { format!("\x1b[33m{}\x1b[0m", s) }
fn dim(s: &str) -> String    { format!("\x1b[2m{}\x1b[0m", s) }
fn bold(s: &str) -> String   { format!("\x1b[1m{}\x1b[0m", s) }

fn urlenc(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' | '/' => c.to_string(),
        ' ' => "+".to_string(),
        c => format!("%{:02X}", c as u32),
    }).collect()
}
