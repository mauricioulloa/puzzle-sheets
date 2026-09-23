use crate::chess::api::ChessApi;
use crate::web::{self, AppState};
use anyhow::{Context, Result};
use clap::Parser;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Parser, Debug)]
#[command(
    name = "puzzle-sheets",
    version,
    about = "Printable puzzle worksheets for students and educators"
)]
pub struct ServeArgs {
    /// Address to listen on
    #[arg(long, env = "BIND_ADDR", default_value = "127.0.0.1:8080")]
    pub bind: String,

    /// Where chess-puzzle-api lives
    #[arg(
        long,
        env = "CHESS_API_URL",
        default_value = "https://chess.mauriulloa.com"
    )]
    pub chess_api_url: String,

    /// API key for chess-puzzle-api. Without one every sheet shares the
    /// anonymous rate limit, and a sheet costs two requests per puzzle.
    #[arg(long, env = "CHESS_API_KEY", hide_env_values = true)]
    pub chess_api_key: Option<String>,

    /// Origin used in links handed to agents
    #[arg(long, env = "PUBLIC_URL", default_value = "http://localhost:8080")]
    pub public_url: String,

    /// Public hostnames the MCP endpoint should answer to, comma separated.
    ///
    /// The MCP transport refuses Host headers it does not recognise, which is
    /// DNS-rebinding protection. It knows the loopback names already, so a
    /// deployment on a real domain has to name itself here or /mcp returns
    /// 403 to every caller.
    #[arg(long, env = "MCP_ALLOWED_HOSTS", value_delimiter = ',')]
    pub mcp_allowed_hosts: Vec<String>,
}

pub async fn run(args: ServeArgs) -> Result<()> {
    if args.chess_api_key.is_none() {
        tracing::warn!("no CHESS_API_KEY: sheets share the anonymous rate limit");
    }
    let state = Arc::new(AppState {
        api: ChessApi::new(&args.chess_api_url, args.chess_api_key),
        public_url: args.public_url.trim_end_matches('/').to_string(),
    });
    let app = web::router(state, &args.mcp_allowed_hosts);

    let listener = TcpListener::bind(&args.bind)
        .await
        .with_context(|| format!("binding {}", args.bind))?;
    tracing::info!(
        "listening on http://{}, puzzles from {}",
        listener.local_addr()?,
        args.chess_api_url
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("serving")
}

/// Waits for either interactive interruption or the signal an orchestrator
/// actually sends on every deploy.
async fn shutdown_signal() {
    let interrupt = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(err) => {
                tracing::warn!("cannot listen for SIGTERM: {err}");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = interrupt => {},
        () = terminate => {},
    }
    tracing::info!("shutting down");
}
