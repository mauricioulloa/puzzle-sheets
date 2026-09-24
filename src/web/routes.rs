use crate::chess::api::ChessApi;
use crate::mcp::SheetTools;
use crate::web::handlers;
use axum::Router;
use axum::routing::get;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::streamable_http_server::tower::StreamableHttpServerConfig;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

pub struct AppState {
    pub api: ChessApi,
    /// Absolute origin for links handed to agents, which cannot resolve a
    /// relative path.
    pub public_url: String,
}

pub type SharedState = Arc<AppState>;

/// `mcp_allowed_hosts` are the public hostnames `/mcp` answers to, beyond the
/// loopback names rmcp allows out of the box (see `ServeArgs`).
pub fn router(state: SharedState, mcp_allowed_hosts: &[String]) -> Router {
    // Stateless, like chess-puzzle-api's: no sessions to keep, nothing lost
    // on a restart.
    let mut mcp_config = StreamableHttpServerConfig::default();
    mcp_config.legacy_session_mode = false;
    mcp_config.json_response = true;
    mcp_config
        .allowed_hosts
        .extend(mcp_allowed_hosts.iter().cloned());
    let mcp_state = Arc::clone(&state);
    let mcp = StreamableHttpService::new(
        move || Ok(SheetTools::new(Arc::clone(&mcp_state))),
        Arc::new(NeverSessionManager::default()),
        mcp_config,
    );

    Router::new()
        .route("/", get(handlers::landing))
        .route("/sheet/new", get(handlers::new_sheet))
        .route("/sheet", get(handlers::sheet))
        .route("/health", get(handlers::health))
        .route("/llms.txt", get(handlers::llms_txt))
        .nest_service("/mcp", mcp)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
}
