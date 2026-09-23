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
            WorksheetError::Invalid(format!("`answers` must be page or none; got `{value}`."))
        }),
    }
}

/// An error page in the language the visitor asked for.
fn failure(err: WorksheetError, lang: Lang) -> Response {
    let (status, message) = match err {
        WorksheetError::Invalid(message) => (StatusCode::BAD_REQUEST, message),
        WorksheetError::Upstream(err) => {
            tracing::error!("chess-puzzle-api failed: {err:#}");
            (
                StatusCode::BAD_GATEWAY,
                lang.text().service_down.to_string(),
            )
        }
    };
    (status, Html(pages::error(&message, lang))).into_response()
}

#[derive(Deserialize)]
pub struct LandingParams {
    lang: Option<String>,
}

pub async fn landing(Query(params): Query<LandingParams>, headers: HeaderMap) -> Html<String> {
    Html(pages::landing(lang_for(params.lang.as_deref(), &headers)))
}

#[derive(Deserialize)]
pub struct NewSheetParams {
    level: Option<String>,
    theme: Option<String>,
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
) -> Response {
    let lang = lang_for(params.lang.as_deref(), &headers);
    let created = async {
        let request = worksheet::Request {
            level: params.level,
            theme: params.theme,
            count: params.count,
            lang,
            answers: parse_answers(params.answers.as_deref())?,
            title: params.title.filter(|title| !title.trim().is_empty()),
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
    ids: String,
    level: Option<String>,
    theme: Option<String>,
    lang: Option<String>,
    answers: Option<String>,
    title: Option<String>,
}

pub async fn sheet(
    State(state): State<SharedState>,
    Query(params): Query<SheetParams>,
    headers: HeaderMap,
) -> Response {
    let lang = lang_for(params.lang.as_deref(), &headers);
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
    let ids = worksheet::parse_ids(&params.ids)?;
    let answers = parse_answers(params.answers.as_deref())?;
    let level = worksheet::find_level(params.level.as_deref())?;
    let theme = worksheet::find_theme(params.theme.as_deref())?;
    let title = params
        .title
        .unwrap_or_else(|| lang.text().default_title.to_string());
    worksheet::check_title(&title)?;

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
