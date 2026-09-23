//! Turning a request into a worksheet: shared by the web form and the MCP
//! tools, so both make exactly the same sheets.
//!
//! A sheet is identified by its URL, which lists the puzzle ids. No sheet is
//! ever stored: the link reprints the same puzzles and the same solutions.

use crate::chess::api::{ChessApi, ChessApiError, Filter};
use crate::chess::options::{self, Level, THEMES, Theme};
use crate::i18n::Lang;
use crate::sheet::{Answers, MAX_ITEMS, PER_PAGE};

const MAX_TITLE: usize = 80;

#[derive(Debug, thiserror::Error)]
pub enum WorksheetError {
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Upstream(anyhow::Error),
}

impl From<ChessApiError> for WorksheetError {
    fn from(err: ChessApiError) -> Self {
        match err {
            ChessApiError::Rejected(message) => Self::Invalid(message),
            ChessApiError::Failed(err) => Self::Upstream(err),
        }
    }
}

#[derive(Debug, Default)]
pub struct Request {
    pub level: Option<String>,
    /// A theme id; none means any theme.
    pub theme: Option<String>,
    pub count: Option<usize>,
    pub lang: Lang,
    pub answers: Answers,
    pub title: Option<String>,
}

pub struct Created {
    /// Path and query of the printable sheet, e.g. `/sheet?ids=...`.
    pub path: String,
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
    let level = find_level(request.level.as_deref())?;
    let theme = find_theme(request.theme.as_deref())?;

    let filter = Filter {
        themes: theme
            .map(|theme| theme.id.to_string())
            .into_iter()
            .collect(),
        rating_min: level.rating_min,
        rating_max: level.rating_max,
    };
    let puzzle_ids = draw(api, &filter, count).await?;
    if puzzle_ids.is_empty() {
        // The API's own wording names ratings and piece counts; a teacher
        // picked a level and a theme, so say it in those terms.
        return Err(WorksheetError::Invalid(
            request.lang.text().no_puzzles.to_string(),
        ));
    }

    Ok(Created {
        path: sheet_path(&SheetAddress {
            ids: &puzzle_ids,
            level,
            theme,
            lang: request.lang,
            answers: request.answers,
            title: request.title.as_deref(),
        }),
        puzzle_ids,
    })
}

/// Up to `count` puzzle ids; an empty slice of the dataset is just none.
async fn draw(
    api: &ChessApi,
    filter: &Filter,
    count: usize,
) -> Result<Vec<String>, WorksheetError> {
    match api.random(filter, count).await {
        Ok(puzzles) => Ok(puzzles.into_iter().map(|puzzle| puzzle.id).collect()),
        Err(ChessApiError::Rejected(_)) => Ok(Vec::new()),
        Err(err) => Err(err.into()),
    }
}

pub fn find_level(id: Option<&str>) -> Result<&'static Level, WorksheetError> {
    let id = id.unwrap_or(options::DEFAULT_LEVEL);
    options::level(id).ok_or_else(|| {
        let known: Vec<&str> = options::LEVELS.iter().map(|level| level.id).collect();
        WorksheetError::Invalid(format!("Unknown level `{id}`. Known: {}", known.join(", ")))
    })
}

pub fn find_theme(id: Option<&str>) -> Result<Option<&'static Theme>, WorksheetError> {
    match id.filter(|id| !id.is_empty()) {
        None => Ok(None),
        Some(id) => options::theme(id).map(Some).ok_or_else(|| {
            let known: Vec<&str> = THEMES.iter().map(|theme| theme.id).collect();
            WorksheetError::Invalid(format!("Unknown theme `{id}`. Known: {}", known.join(", ")))
        }),
    }
}

/// Everything a sheet's URL carries.
pub struct SheetAddress<'a> {
    pub ids: &'a [String],
    pub level: &'a Level,
    pub theme: Option<&'a Theme>,
    pub lang: Lang,
    pub answers: Answers,
    pub title: Option<&'a str>,
}

pub fn sheet_path(address: &SheetAddress) -> String {
    let ids = address.ids.join(",");
    let mut pairs = vec![
        ("ids", ids.as_str()),
        ("level", address.level.id),
        ("lang", address.lang.code()),
        ("answers", address.answers.code()),
    ];
    if let Some(theme) = address.theme {
        pairs.push(("theme", theme.id));
    }
    if let Some(title) = address.title {
        pairs.push(("title", title));
    }
    let query = serde_urlencoded::to_string(pairs).expect("strings always encode");
    format!("/sheet?{query}")
}

/// "Difficulty: Novice (1000–1399) · Theme: Fork", in the sheet's language.
pub fn subtitle(level: &Level, theme: Option<&Theme>, lang: Lang) -> String {
    let text = lang.text();
    let mut subtitle = format!(
        "{}: {} ({})",
        text.difficulty,
        level.label.get(lang),
        level.range()
    );
    if let Some(theme) = theme {
        subtitle.push_str(&format!(" · {}: {}", text.theme, theme.label.get(lang)));
    }
    subtitle
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
    fn a_level_is_required_to_exist_but_defaults_sensibly() {
        assert_eq!(find_level(None).unwrap().id, options::DEFAULT_LEVEL);
        assert!(find_level(Some("grandmaster")).is_err());
    }

    #[test]
    fn no_theme_means_any_theme() {
        assert!(find_theme(None).unwrap().is_none());
        assert!(find_theme(Some("")).unwrap().is_none());
        assert_eq!(find_theme(Some("fork")).unwrap().unwrap().id, "fork");
        assert!(find_theme(Some("chessboxing")).is_err());
    }

    #[test]
    fn the_subtitle_reads_in_the_sheets_language() {
        let novice = options::level("novice").unwrap();
        let fork = options::theme("fork");
        assert_eq!(
            subtitle(novice, fork, Lang::Es),
            "Dificultad: Inicial (1000–1399) · Tema: Ataque doble"
        );
        assert_eq!(
            subtitle(novice, None, Lang::En),
            "Difficulty: Novice (1000–1399)"
        );
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
    fn the_sheet_path_carries_everything_needed_to_reprint() {
        let path = sheet_path(&SheetAddress {
            ids: &["a".into(), "b".into()],
            level: options::level("beginner").unwrap(),
            theme: options::theme("pin"),
            lang: Lang::Es,
            answers: Answers::None,
            title: Some("3º básico & más"),
        });
        assert!(
            path.starts_with(
                "/sheet?ids=a%2Cb&level=beginner&lang=es&answers=none&theme=pin&title="
            ),
            "{path}"
        );
        assert_eq!(
            path.matches('&').count(),
            5,
            "the & in the title is encoded"
        );
    }
}
