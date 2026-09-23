//! Turning a request into a worksheet: shared by the web form and the MCP
//! tools, so both make exactly the same sheets.
//!
//! A sheet is identified by its URL, which lists the puzzle ids. No sheet is
//! ever stored: the link reprints the same puzzles and the same answer key.

use crate::chess::api::{ApiError, ChessApi, Filter};
use crate::chess::presets::{self, Preset};
use crate::i18n::Lang;
use crate::sheet::{Answers, MAX_ITEMS, PER_PAGE};

const MAX_TITLE: usize = 80;
/// How far either side of a requested rating a custom sheet searches.
const RATING_TOLERANCE: u32 = 150;
const RATING_CEILING: u32 = 3500;

#[derive(Debug, thiserror::Error)]
pub enum WorksheetError {
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Upstream(anyhow::Error),
}

impl From<ApiError> for WorksheetError {
    fn from(err: ApiError) -> Self {
        match err {
            ApiError::Rejected(message) => Self::Invalid(message),
            ApiError::Failed(err) => Self::Upstream(err),
        }
    }
}

#[derive(Debug, Default)]
pub struct Request {
    pub preset: Option<String>,
    pub themes: Vec<String>,
    pub rating: Option<u32>,
    pub max_pieces: Option<u32>,
    pub count: Option<usize>,
    pub lang: Lang,
    pub answers: Answers,
    pub title: Option<String>,
}

pub struct Created {
    /// Path and query of the printable sheet, e.g. `/sheet?ids=...`.
    pub path: String,
    pub title: String,
    pub puzzle_ids: Vec<String>,
}

pub async fn create(api: &ChessApi, request: Request) -> Result<Created, WorksheetError> {
    let count = request.count.unwrap_or(PER_PAGE);
    if !(1..=MAX_ITEMS).contains(&count) {
        return Err(WorksheetError::Invalid(format!(
            "count must be between 1 and {MAX_ITEMS}"
        )));
    }

    if let Some(title) = &request.title {
        check_title(title)?;
    }
    let preset = match request.preset.as_deref() {
        Some(id) => Some(presets::find(id).ok_or_else(|| {
            WorksheetError::Invalid(format!("Unknown preset `{id}`. Known: {}", preset_ids()))
        })?),
        None => None,
    };
    let filter = filter_for(preset, &request)?;
    let title = match &request.title {
        Some(title) => title.clone(),
        None => preset
            .map_or(request.lang.text().default_title, |preset| {
                preset.title(request.lang)
            })
            .to_string(),
    };

    let puzzle_ids: Vec<String> = api
        .random(&filter, count)
        .await?
        .into_iter()
        .map(|puzzle| puzzle.id)
        .collect();

    Ok(Created {
        path: sheet_path(&puzzle_ids, request.lang, request.answers, &title),
        title,
        puzzle_ids,
    })
}

/// A preset supplies the defaults; anything the caller names overrides it.
fn filter_for(preset: Option<&Preset>, request: &Request) -> Result<Filter, WorksheetError> {
    let themes = if request.themes.is_empty() {
        preset
            .map(|preset| {
                preset
                    .themes
                    .iter()
                    .map(|theme| theme.to_string())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        request.themes.clone()
    };
    if themes.is_empty() {
        return Err(WorksheetError::Invalid(format!(
            "Choose a preset ({}) or name at least one theme.",
            preset_ids()
        )));
    }

    let (rating_min, rating_max) = match (request.rating, preset) {
        (Some(rating), _) => (
            rating.saturating_sub(RATING_TOLERANCE),
            (rating + RATING_TOLERANCE).min(RATING_CEILING),
        ),
        (None, Some(preset)) => (preset.rating_min, preset.rating_max),
        (None, None) => (0, RATING_CEILING),
    };

    Ok(Filter {
        themes,
        rating_min,
        rating_max,
        max_pieces: request
            .max_pieces
            .or(preset.map(|preset| preset.max_pieces)),
    })
}

fn preset_ids() -> String {
    presets::PRESETS
        .iter()
        .map(|preset| preset.id)
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn sheet_path(ids: &[String], lang: Lang, answers: Answers, title: &str) -> String {
    let query = serde_urlencoded::to_string([
        ("ids", ids.join(",").as_str()),
        ("lang", lang.code()),
        ("answers", answers.code()),
        ("title", title),
    ])
    .expect("strings always encode");
    format!("/sheet?{query}")
}

/// Puzzle ids go into upstream URLs, so only plain alphanumerics pass.
pub fn parse_ids(raw: &str) -> Result<Vec<String>, WorksheetError> {
    let ids: Vec<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect();
    if ids.is_empty() || ids.len() > MAX_ITEMS {
        return Err(WorksheetError::Invalid(format!(
            "A sheet holds 1 to {MAX_ITEMS} puzzles."
        )));
    }
    if let Some(bad) = ids
        .iter()
        .find(|id| id.len() > 16 || !id.chars().all(|ch| ch.is_ascii_alphanumeric()))
    {
        return Err(WorksheetError::Invalid(format!(
            "`{bad}` is not a puzzle id."
        )));
    }
    Ok(ids)
}

pub fn check_title(title: &str) -> Result<(), WorksheetError> {
    if title.chars().count() > MAX_TITLE {
        return Err(WorksheetError::Invalid(format!(
            "The title can be at most {MAX_TITLE} characters."
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_preset_supplies_defaults_the_caller_can_override() {
        let forks = presets::find("forks").unwrap();
        let request = Request {
            max_pieces: Some(10),
            ..Default::default()
        };
        let filter = filter_for(Some(forks), &request).unwrap();
        assert_eq!(filter.themes, ["fork"]);
        assert_eq!((filter.rating_min, filter.rating_max), (600, 1200));
        assert_eq!(filter.max_pieces, Some(10));
    }

    #[test]
    fn a_custom_rating_becomes_a_band() {
        let request = Request {
            themes: vec!["pin".into()],
            rating: Some(100),
            ..Default::default()
        };
        let filter = filter_for(None, &request).unwrap();
        assert_eq!((filter.rating_min, filter.rating_max), (0, 250));
        assert_eq!(filter.max_pieces, None);
    }

    #[test]
    fn nothing_to_go_on_is_an_error() {
        assert!(filter_for(None, &Request::default()).is_err());
    }

    #[test]
    fn ids_are_limited_to_what_a_sheet_holds_and_what_is_safe() {
        assert_eq!(parse_ids("00008, 00014").unwrap(), ["00008", "00014"]);
        assert!(parse_ids("").is_err());
        assert!(parse_ids(&vec!["a"; 13].join(",")).is_err());
        assert!(parse_ids("../admin").is_err());
        assert!(parse_ids("00008?x=1").is_err());
    }

    #[test]
    fn the_sheet_path_round_trips_a_title() {
        let path = sheet_path(
            &["a".into(), "b".into()],
            Lang::Es,
            Answers::Footer,
            "3º básico & más",
        );
        assert!(path.starts_with("/sheet?ids=a%2Cb&lang=es&answers=footer&title="));
        assert_eq!(
            path.matches('&').count(),
            3,
            "the & in the title is encoded"
        );
    }
}
