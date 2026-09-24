use crate::chess::{self, board};
use crate::i18n::Lang;
use crate::sheet::{self, Sheet};
use crate::web::pages;
use crate::web::routes::SharedState;
use crate::worksheet::{self, Invalid, WorksheetError};
use axum::Json;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};

/// What to suggest when chess-puzzle-api is busy and does not say for how
/// long.
const DEFAULT_RETRY_SECS: u64 = 60;

fn browser_lang(headers: &HeaderMap) -> Lang {
    Lang::from_accept_language(
        headers
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok()),
    )
}

/// The language a sheet asked for, or the browser's. An unknown `lang` is
/// refused rather than guessed at, the same as over MCP.
fn sheet_lang(requested: Option<&str>, headers: &HeaderMap) -> Result<Lang, Invalid> {
    Ok(worksheet::parse_lang(requested)?.unwrap_or_else(|| browser_lang(headers)))
}

/// An error page in the visitor's language.
fn failure(err: WorksheetError, lang: Lang) -> Response {
    let text = lang.text();
    let (status, message, retry_after) = match err {
        WorksheetError::Invalid(invalid) => (StatusCode::BAD_REQUEST, invalid.message(lang), None),
        WorksheetError::Busy { retry_after } => {
            let seconds = retry_after.unwrap_or(DEFAULT_RETRY_SECS);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("{} {seconds} {}.", text.service_busy, text.seconds),
                Some(seconds),
            )
        }
        WorksheetError::Upstream(err) => {
            tracing::error!("chess-puzzle-api failed: {err:#}");
            (StatusCode::BAD_GATEWAY, text.service_down.to_string(), None)
        }
    };
    let mut response = (status, Html(pages::error(&message, lang))).into_response();
    if let Some(seconds) = retry_after {
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from(seconds));
    }
    response
}

/// The query string itself could not be read, e.g. a repeated parameter.
fn unreadable(rejection: &QueryRejection, headers: &HeaderMap) -> Response {
    tracing::debug!("unreadable query: {rejection}");
    failure(Invalid::Query.into(), browser_lang(headers))
}

#[derive(Deserialize)]
pub struct LandingParams {
    lang: Option<String>,
}

/// The front door is lenient: an unknown `lang` just falls back to the
/// browser's.
pub async fn landing(
    params: Result<Query<LandingParams>, QueryRejection>,
    headers: HeaderMap,
) -> Html<String> {
    let requested = params.ok().and_then(|Query(params)| params.lang);
    let lang = requested
        .as_deref()
        .and_then(Lang::parse)
        .unwrap_or_else(|| browser_lang(&headers));
    Html(pages::landing(lang))
}

/// Every field is text, so a malformed value is refused in words a teacher
/// can read rather than by the extractor.
#[derive(Deserialize)]
pub struct NewSheetParams {
    level: Option<String>,
    theme: Option<String>,
    count: Option<String>,
    lang: Option<String>,
    answers: Option<String>,
    title: Option<String>,
}

/// Picks the puzzles, then redirects to the sheet's permanent address so the
/// page a teacher prints is one they can print again.
pub async fn new_sheet(
    State(state): State<SharedState>,
    params: Result<Query<NewSheetParams>, QueryRejection>,
    headers: HeaderMap,
) -> Response {
    let Query(params) = match params {
        Ok(params) => params,
        Err(rejection) => return unreadable(&rejection, &headers),
    };
    let lang = match sheet_lang(params.lang.as_deref(), &headers) {
        Ok(lang) => lang,
        Err(invalid) => return failure(invalid.into(), browser_lang(&headers)),
    };
    let created = async {
        let request = worksheet::Request {
            level: params.level,
            theme: params.theme,
            count: worksheet::parse_count(params.count.as_deref())?,
            lang,
            answers: worksheet::parse_answers(params.answers.as_deref())?,
            title: params.title,
        };
        worksheet::create(&state.api, request).await
    };
    match created.await {
        Ok(created) => Redirect::to(&created.path).into_response(),
        Err(err) => failure(err, lang),
    }
}

#[derive(Deserialize)]
pub struct SheetParams {
    ids: Option<String>,
    level: Option<String>,
    theme: Option<String>,
    lang: Option<String>,
    answers: Option<String>,
    title: Option<String>,
}

pub async fn sheet(
    State(state): State<SharedState>,
    params: Result<Query<SheetParams>, QueryRejection>,
    headers: HeaderMap,
) -> Response {
    let Query(params) = match params {
        Ok(params) => params,
        Err(rejection) => return unreadable(&rejection, &headers),
    };
    let lang = match sheet_lang(params.lang.as_deref(), &headers) {
        Ok(lang) => lang,
        Err(invalid) => return failure(invalid.into(), browser_lang(&headers)),
    };
    match render_sheet(&state, params, lang).await {
        Ok(html) => Html(html).into_response(),
        Err(err) => failure(err, lang),
    }
}

async fn render_sheet(
    state: &SharedState,
    params: SheetParams,
    lang: Lang,
) -> Result<String, WorksheetError> {
    let ids = worksheet::parse_ids(params.ids.as_deref())?;
    let answers = worksheet::parse_answers(params.answers.as_deref())?;
    let level = worksheet::find_level(params.level.as_deref())?;
    let theme = worksheet::find_theme(params.theme.as_deref())?;
    let title = worksheet::clean_title(params.title)?
        .unwrap_or_else(|| lang.text().default_title.to_string());

    let items = chess::items(&state.api, &ids, lang).await?;
    Ok(sheet::render(&Sheet {
        title,
        subtitle: Some(worksheet::subtitle(level, theme, lang)),
        items,
        lang,
        answers,
        defs: board::DEFS,
        again_url: format!("/?lang={}", lang.code()),
    }))
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub async fn llms_txt(State(state): State<SharedState>) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
        pages::llms_txt(&state.public_url),
    )
}
