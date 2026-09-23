use crate::chess::{self, board};
use crate::i18n::Lang;
use crate::sheet::{self, Answers, Sheet};
use crate::web::pages;
use crate::web::routes::SharedState;
use crate::worksheet::{self, WorksheetError};
use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};

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
pub struct LandingParams {
    lang: Option<String>,
}

pub async fn landing(Query(params): Query<LandingParams>, headers: HeaderMap) -> Html<String> {
    Html(pages::landing(lang_for(params.lang.as_deref(), &headers)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSheetParams {
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
pub async fn new_sheet(
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
pub struct SheetParams {
    ids: String,
    lang: Option<String>,
    answers: Option<String>,
    title: Option<String>,
}

pub async fn sheet(
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
