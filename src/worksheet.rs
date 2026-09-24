//! Turning a request into a worksheet: shared by the web form and the MCP
//! tools, so both make exactly the same sheets and refuse the same mistakes
//! in the same words.
//!
//! A sheet is identified by its URL, which lists the puzzle ids. No sheet is
//! ever stored: the link reprints the same puzzles and the same solutions.

use crate::chess::api::{ChessApi, ChessApiError, Filter};
use crate::chess::options::{self, Level, THEMES, Theme};
use crate::i18n::Lang;
use crate::sheet::{Answers, MAX_ITEMS, PER_PAGE};

const MAX_TITLE: usize = 80;
/// Lichess ids are five characters; the slack is for whatever comes next.
const MAX_ID_LEN: usize = 16;

#[derive(Debug, thiserror::Error)]
pub enum WorksheetError {
    #[error("{}", .0.message(Lang::En))]
    Invalid(Invalid),
    /// chess-puzzle-api is over its rate limit or stopped a slow search.
    #[error("the puzzle service is busy")]
    Busy { retry_after: Option<u64> },
    #[error(transparent)]
    Upstream(anyhow::Error),
}

impl From<Invalid> for WorksheetError {
    fn from(invalid: Invalid) -> Self {
        Self::Invalid(invalid)
    }
}

impl From<ChessApiError> for WorksheetError {
    fn from(err: ChessApiError) -> Self {
        match err {
            ChessApiError::Busy { retry_after } => Self::Busy { retry_after },
            // Callers that look a puzzle up by id say which one was missing;
            // anywhere else a 404 is the two services disagreeing.
            ChessApiError::NotFound => {
                Self::Upstream(anyhow::anyhow!("chess-puzzle-api answered 404"))
            }
            ChessApiError::Failed(err) => Self::Upstream(err),
        }
    }
}

/// A request this service refuses, and why. Kept as data rather than text so
/// the web page can say it in the visitor's language; MCP says it in English.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invalid {
    /// The query string could not be read at all.
    Query,
    Count(String),
    Title,
    Level(String),
    Theme(String),
    IdCount,
    Id(String),
    UnknownPuzzle(String),
    Answers(String),
    Lang(String),
    NoPuzzles,
}

impl Invalid {
    pub fn message(&self, lang: Lang) -> String {
        let es = lang == Lang::Es;
        match self {
            Self::Query if es => {
                "No se pudo leer el enlace. Vuelve al formulario y crea la hoja de nuevo.".into()
            }
            Self::Query => {
                "The link could not be read. Go back to the form and make the sheet again.".into()
            }
            Self::Count(raw) if es => {
                format!("La cantidad de ejercicios (`count`) va de 1 a {MAX_ITEMS}; llegó `{raw}`.")
            }
            Self::Count(raw) => {
                format!("`count` must be a number from 1 to {MAX_ITEMS}; got `{raw}`.")
            }
            Self::Title if es => format!("El título puede tener hasta {MAX_TITLE} caracteres."),
            Self::Title => format!("The title can be at most {MAX_TITLE} characters."),
            Self::Level(raw) if es => {
                format!(
                    "No existe el nivel `{raw}`. Los niveles son: {}.",
                    level_ids()
                )
            }
            Self::Level(raw) => format!("Unknown level `{raw}`. Known: {}.", level_ids()),
            Self::Theme(raw) if es => {
                format!("No existe el tema `{raw}`. Los temas son: {}.", theme_ids())
            }
            Self::Theme(raw) => format!("Unknown theme `{raw}`. Known: {}.", theme_ids()),
            Self::IdCount if es => format!("Una hoja lleva de 1 a {MAX_ITEMS} ejercicios."),
            Self::IdCount => format!("A sheet holds 1 to {MAX_ITEMS} puzzles."),
            Self::Id(raw) if es => format!("`{raw}` no es un identificador de ejercicio."),
            Self::Id(raw) => format!("`{raw}` is not a puzzle id."),
            Self::UnknownPuzzle(id) if es => format!("No hay ningún ejercicio `{id}`."),
            Self::UnknownPuzzle(id) => format!("No puzzle with id `{id}`."),
            Self::Answers(raw) if es => format!("`answers` debe ser page o none; llegó `{raw}`."),
            Self::Answers(raw) => format!("`answers` must be page or none; got `{raw}`."),
            Self::Lang(raw) if es => format!("`lang` debe ser es o en; llegó `{raw}`."),
            Self::Lang(raw) => format!("`lang` must be es or en; got `{raw}`."),
            Self::NoPuzzles => lang.text().no_puzzles.to_string(),
        }
    }
}

fn level_ids() -> String {
    let ids: Vec<&str> = options::LEVELS.iter().map(|level| level.id).collect();
    ids.join(", ")
}

fn theme_ids() -> String {
    let ids: Vec<&str> = THEMES.iter().map(|theme| theme.id).collect();
    ids.join(", ")
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
        return Err(Invalid::Count(count.to_string()).into());
    }
    let title = clean_title(request.title)?;
    let level = find_level(request.level.as_deref())?;
    let theme = find_theme(request.theme.as_deref())?;

    let filter = Filter {
        theme: theme.map(|theme| theme.id),
        rating_min: level.rating_min,
        rating_max: level.rating_max,
    };
    let puzzle_ids = draw(api, &filter, count).await?;
    if puzzle_ids.is_empty() {
        // The API's own wording names ratings and piece counts; a teacher
        // picked a level and a theme, so say it in those terms.
        return Err(Invalid::NoPuzzles.into());
    }

    Ok(Created {
        path: sheet_path(&SheetAddress {
            ids: &puzzle_ids,
            level,
            theme,
            lang: request.lang,
            answers: request.answers,
            title: title.as_deref(),
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
    let puzzles = api.random(filter, count).await?;
    Ok(puzzles.into_iter().map(|puzzle| puzzle.id).collect())
}

pub fn find_level(id: Option<&str>) -> Result<&'static Level, Invalid> {
    let id = id.unwrap_or(options::DEFAULT_LEVEL);
    options::level(id).ok_or_else(|| Invalid::Level(id.to_string()))
}

pub fn find_theme(id: Option<&str>) -> Result<Option<&'static Theme>, Invalid> {
    match id.filter(|id| !id.is_empty()) {
        None => Ok(None),
        Some(id) => options::theme(id)
            .map(Some)
            .ok_or_else(|| Invalid::Theme(id.to_string())),
    }
}

/// None when the parameter is absent, so each surface picks its own default.
pub fn parse_lang(raw: Option<&str>) -> Result<Option<Lang>, Invalid> {
    raw.map(|value| Lang::parse(value).ok_or_else(|| Invalid::Lang(value.to_string())))
        .transpose()
}

pub fn parse_answers(raw: Option<&str>) -> Result<Answers, Invalid> {
    match raw {
        None => Ok(Answers::default()),
        Some(value) => Answers::parse(value).ok_or_else(|| Invalid::Answers(value.to_string())),
    }
}

/// `count` as the web form sends it, text; range is checked by `create`.
pub fn parse_count(raw: Option<&str>) -> Result<Option<usize>, Invalid> {
    raw.map(|value| {
        value
            .trim()
            .parse()
            .map_err(|_| Invalid::Count(value.to_string()))
    })
    .transpose()
}

/// A blank title is no title, whichever surface it came through; anything
/// else is kept as typed, less the spaces around it.
pub fn clean_title(raw: Option<String>) -> Result<Option<String>, Invalid> {
    let Some(title) = raw else { return Ok(None) };
    let title = title.trim();
    if title.is_empty() {
        return Ok(None);
    }
    if title.chars().count() > MAX_TITLE {
        return Err(Invalid::Title);
    }
    Ok(Some(title.to_string()))
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
        level.range(lang)
    );
    if let Some(theme) = theme {
        subtitle.push_str(&format!(" · {}: {}", text.theme, theme.label.get(lang)));
    }
    subtitle
}

/// Puzzle ids go into upstream URLs, so only plain alphanumerics pass.
pub fn parse_ids(raw: Option<&str>) -> Result<Vec<String>, Invalid> {
    let ids: Vec<String> = raw
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect();
    if ids.is_empty() || ids.len() > MAX_ITEMS {
        return Err(Invalid::IdCount);
    }
    if let Some(bad) = ids
        .iter()
        .find(|id| id.len() > MAX_ID_LEN || !id.chars().all(|ch| ch.is_ascii_alphanumeric()))
    {
        return Err(Invalid::Id(bad.clone()));
    }
    Ok(ids)
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
    fn a_blank_title_is_no_title() {
        assert_eq!(clean_title(None), Ok(None));
        assert_eq!(clean_title(Some("   ".into())), Ok(None));
        assert_eq!(
            clean_title(Some(" 3º básico ".into())),
            Ok(Some("3º básico".into()))
        );
        assert_eq!(clean_title(Some("x".repeat(81))), Err(Invalid::Title));
    }

    #[test]
    fn form_values_parse_or_say_what_was_wrong() {
        assert_eq!(parse_count(Some("12")), Ok(Some(12)));
        assert_eq!(parse_count(Some("abc")), Err(Invalid::Count("abc".into())));
        assert_eq!(parse_lang(None), Ok(None));
        assert_eq!(parse_lang(Some("EN")), Ok(Some(Lang::En)));
        assert_eq!(parse_lang(Some("fr")), Err(Invalid::Lang("fr".into())));
        assert_eq!(
            parse_answers(Some("footer")),
            Err(Invalid::Answers("footer".into()))
        );
    }

    #[test]
    fn every_refusal_reads_in_both_languages() {
        let cases = [
            Invalid::Query,
            Invalid::Count("x".into()),
            Invalid::Title,
            Invalid::Level("x".into()),
            Invalid::Theme("x".into()),
            Invalid::IdCount,
            Invalid::Id("x".into()),
            Invalid::UnknownPuzzle("x".into()),
            Invalid::Answers("x".into()),
            Invalid::Lang("x".into()),
            Invalid::NoPuzzles,
        ];
        for invalid in cases {
            let (es, en) = (invalid.message(Lang::Es), invalid.message(Lang::En));
            assert!(!es.is_empty() && !en.is_empty());
            assert_ne!(es, en, "{invalid:?} is not translated");
        }
        assert!(
            Invalid::Theme("x".into())
                .message(Lang::Es)
                .contains("zugzwang")
        );
    }

    #[test]
    fn ids_are_limited_to_what_a_sheet_holds_and_what_is_safe() {
        assert_eq!(parse_ids(Some("00008, 00014")).unwrap(), ["00008", "00014"]);
        assert_eq!(parse_ids(None), Err(Invalid::IdCount));
        assert_eq!(parse_ids(Some("")), Err(Invalid::IdCount));
        assert!(parse_ids(Some(&vec!["a"; 13].join(","))).is_err());
        assert_eq!(
            parse_ids(Some("../admin")),
            Err(Invalid::Id("../admin".into()))
        );
        assert!(parse_ids(Some("00008?x=1")).is_err());
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
