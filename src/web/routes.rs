use crate::chess::api::ChessApi;
use crate::mcp::SheetTools;
use crate::web::handlers;
use axum::Router;
use axum::http::{HeaderName, HeaderValue, header};
use axum::routing::get;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::streamable_http_server::tower::StreamableHttpServerConfig;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

/// Everything is served from here and inline: no fonts, images or scripts
/// from elsewhere. Styles are inline throughout; the one script, the print
/// button's, is allowed by its hash (`sheet::PRINT_SCRIPT`). Nothing may
/// frame a page, and the form may only submit here.
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'none'; \
     style-src 'unsafe-inline'; \
     script-src 'sha256-oYck0c9qZYrrxfvezOkNwW1dNWvK66v0jMpd+ak28kE='; \
     img-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'";

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
        .layer(header_layer(
            header::CONTENT_SECURITY_POLICY,
            CONTENT_SECURITY_POLICY,
        ))
        .layer(header_layer(header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        // A sheet's URL can carry a class name in its title; a link out of
        // the page should say only which site it came from.
        .layer(header_layer(
            header::REFERRER_POLICY,
            "strict-origin-when-cross-origin",
        ))
}

fn header_layer(name: HeaderName, value: &'static str) -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::if_not_present(name, HeaderValue::from_static(value))
}
