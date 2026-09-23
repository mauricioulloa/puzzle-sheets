//! The HTTP surface: a form, the printable sheet, and the MCP endpoint.

mod pages;

use crate::chess::{self, api::ChessApi, board};
use crate::i18n::Lang;
use crate::mcp::SheetTools;
use crate::sheet::{self, Answers, Sheet};
use crate::worksheet::{self, WorksheetError};
use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::streamable_http_server::tower::StreamableHttpServerConfig;
use serde::Deserialize;
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
        .route("/", get(landing))
        .route("/sheet/new", get(new_sheet))
        .route("/sheet", get(sheet))
        .route("/health", get(health))
        .route("/llms.txt", get(llms_txt))
        .nest_service("/mcp", mcp)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
}

fn lang_for(requested: Option<&str>, headers: &HeaderMap) -> Lang {
    requested.and_then(Lang::parse).unwrap_or_else(|| {
        Lang::from_accept_language(
            headers
                .get(header::ACCEPT_LANGUAGE)
                .and_then(|value| value.to_str().ok()),
        )
    })
}

fn parse_answers(raw: Option<&str>) -> Result<Answers, WorksheetError> {
    match raw {
        None => Ok(Answers::default()),
        Some(value) => Answers::parse(value).ok_or_else(|| {
            WorksheetError::Invalid(format!(
                "`answers` must be page, footer or none; got `{value}`."
            ))
        }),
    }
}

impl IntoResponse for WorksheetError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            WorksheetError::Invalid(message) => (StatusCode::BAD_REQUEST, message),
            WorksheetError::Upstream(err) => {
                tracing::error!("chess-puzzle-api failed: {err:#}");
                (
                    StatusCode::BAD_GATEWAY,
                    "The puzzle service did not answer. Try again in a moment.".to_string(),
                )
            }
        };
        (status, Html(pages::error(&message))).into_response()
    }
}

#[derive(Deserialize)]
struct LandingParams {
    lang: Option<String>,
}

async fn landing(Query(params): Query<LandingParams>, headers: HeaderMap) -> Html<String> {
    Html(pages::landing(lang_for(params.lang.as_deref(), &headers)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewSheetParams {
    preset: Option<String>,
    themes: Option<String>,
    rating: Option<u32>,
    max_pieces: Option<u32>,
    count: Option<usize>,
    lang: Option<String>,
    answers: Option<String>,
    title: Option<String>,
}

/// Picks the puzzles, then redirects to the sheet's permanent address so the
/// page a teacher prints is one they can print again.
async fn new_sheet(
    State(state): State<SharedState>,
    Query(params): Query<NewSheetParams>,
    headers: HeaderMap,
) -> Result<Redirect, WorksheetError> {
    let request = worksheet::Request {
        preset: params.preset.filter(|preset| !preset.is_empty()),
        themes: params
            .themes
            .as_deref()
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|theme| !theme.is_empty())
            .map(str::to_string)
            .collect(),
        rating: params.rating,
        max_pieces: params.max_pieces,
        count: params.count,
        lang: lang_for(params.lang.as_deref(), &headers),
        answers: parse_answers(params.answers.as_deref())?,
        title: params.title.filter(|title| !title.trim().is_empty()),
    };
    let created = worksheet::create(&state.api, request).await?;
    Ok(Redirect::to(&created.path))
}

#[derive(Deserialize)]
struct SheetParams {
    ids: String,
    lang: Option<String>,
    answers: Option<String>,
    title: Option<String>,
}

async fn sheet(
    State(state): State<SharedState>,
    Query(params): Query<SheetParams>,
    headers: HeaderMap,
) -> Result<Html<String>, WorksheetError> {
    let ids = worksheet::parse_ids(&params.ids)?;
    let lang = lang_for(params.lang.as_deref(), &headers);
    let answers = parse_answers(params.answers.as_deref())?;
    let title = params
        .title
        .unwrap_or_else(|| lang.text().default_title.to_string());
    worksheet::check_title(&title)?;

    let items = chess::items(&state.api, &ids, lang).await?;
    Ok(Html(sheet::render(&Sheet {
        title,
        items,
        lang,
        answers,
        defs: board::DEFS,
        again_url: format!("/?lang={}", lang.code()),
    })))
}

async fn health() -> &'static str {
    "ok"
}

async fn llms_txt(State(state): State<SharedState>) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
        pages::llms_txt(&state.public_url),
    )
}
