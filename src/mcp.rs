//! A Model Context Protocol server, so an assistant helping a teacher, a
//! parent or a student can hand them a printable sheet instead of describing
//! positions in chat.

use crate::chess::options::{LEVELS, THEMES};
use crate::i18n::Lang;
use crate::web::routes::SharedState;
use crate::worksheet::{self, Invalid, WorksheetError};
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{ErrorData, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, schemars, tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListOptionsArgs {
    /// `es` or `en`, for the labels. Defaults to `es`.
    pub lang: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct LevelInfo {
    pub id: String,
    pub label: String,
    /// Lichess puzzle ratings, inclusive. Levels are contiguous: every
    /// rating belongs to exactly one.
    pub rating_min: u32,
    pub rating_max: u32,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ThemeInfo {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Options {
    pub levels: Vec<LevelInfo>,
    pub themes: Vec<ThemeInfo>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateWorksheetArgs {
    /// A level id from `list_options`, from `beginner` (under 1000) to
    /// `expert` (2200+), each a band of Lichess puzzle rating. Defaults to
    /// `novice`, 1000–1399.
    pub level: Option<String>,
    /// A theme id from `list_options`, e.g. `fork` or `mateIn2`. Leave it out
    /// for any theme.
    pub theme: Option<String>,
    /// How many puzzles, 1 to 12. Six fill a page. Defaults to 6.
    pub count: Option<usize>,
    /// `es` or `en`. Defaults to `es`.
    pub lang: Option<String>,
    /// `page` (default) puts the solutions on a page of their own after each
    /// page of puzzles, to print on the back; `none` leaves them out, for a
    /// student working alone.
    pub answers: Option<String>,
    /// Heading printed on the sheet, e.g. a class name.
    pub title: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Worksheet {
    /// Open and print this. It is permanent: it always shows these puzzles.
    pub worksheet_url: String,
    pub puzzle_ids: Vec<String>,
}

/// Agents read English most reliably, so refusals are in English whatever
/// language the sheet is in.
impl From<Invalid> for ErrorData {
    fn from(invalid: Invalid) -> Self {
        ErrorData::invalid_params(invalid.message(Lang::En), None)
    }
}

impl From<WorksheetError> for ErrorData {
    fn from(err: WorksheetError) -> Self {
        match err {
            WorksheetError::Invalid(invalid) => invalid.into(),
            WorksheetError::Busy { retry_after } => {
                let when = retry_after.map_or_else(
                    || "in a minute".to_string(),
                    |seconds| format!("in {seconds}s"),
                );
                ErrorData::internal_error(
                    format!("The puzzle service is busy. Try again {when}."),
                    None,
                )
            }
            WorksheetError::Upstream(err) => {
                tracing::error!("mcp tool failed: {err:#}");
                ErrorData::internal_error("The puzzle service failed to answer.", None)
            }
        }
    }
}

#[derive(Clone)]
pub struct SheetTools {
    state: SharedState,
}

#[tool_router]
impl SheetTools {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    #[tool(
        name = "list_options",
        description = "List what a chess worksheet can be made of: difficulty levels, each a \
                       band of Lichess puzzle rating, and tactical themes such as forks or mate in two. \
                       Call this before create_worksheet."
    )]
    async fn list_options(
        &self,
        Parameters(args): Parameters<ListOptionsArgs>,
    ) -> Result<Json<Options>, ErrorData> {
        let lang = worksheet::parse_lang(args.lang.as_deref())?.unwrap_or_default();
        Ok(Json(Options {
            levels: LEVELS
                .iter()
                .map(|level| LevelInfo {
                    id: level.id.to_string(),
                    label: level.label.get(lang).to_string(),
                    rating_min: level.rating_min,
                    rating_max: level.rating_max,
                })
                .collect(),
            themes: THEMES
                .iter()
                .map(|theme| ThemeInfo {
                    id: theme.id.to_string(),
                    label: theme.label.get(lang).to_string(),
                })
                .collect(),
        }))
    }

    #[tool(
        name = "create_worksheet",
        description = "Make a printable chess worksheet and return its link. Each puzzle shows \
                       who moves, what to find (e.g. 'Mate in 2') and its FEN; solutions go on \
                       a page of their own unless asked otherwise. Use this \
                       when a teacher, parent or student wants puzzles on paper."
    )]
    async fn create_worksheet(
        &self,
        Parameters(args): Parameters<CreateWorksheetArgs>,
    ) -> Result<Json<Worksheet>, ErrorData> {
        let request = worksheet::Request {
            level: args.level,
            theme: args.theme,
            count: args.count,
            lang: worksheet::parse_lang(args.lang.as_deref())?.unwrap_or_default(),
            answers: worksheet::parse_answers(args.answers.as_deref())?,
            title: args.title,
        };
        let created = worksheet::create(&self.state.api, request).await?;
        Ok(Json(Worksheet {
            worksheet_url: format!("{}{}", self.state.public_url, created.path),
            puzzle_ids: created.puzzle_ids,
        }))
    }
}

#[tool_handler]
impl ServerHandler for SheetTools {
    fn get_info(&self) -> ServerConfig {
        // ServerConfig and Implementation are non-exhaustive, so they are
        // built from Default rather than with a struct literal.
        let mut server_info = Implementation::default();
        server_info.name = "puzzle-sheets".to_string();
        server_info.title = Some("Chess Worksheets".to_string());
        server_info.version = env!("CARGO_PKG_VERSION").to_string();
        server_info.description = Some(
            "Printable chess worksheets for students and educators, in Spanish and English. \
             Built by Mauri Ulloa."
                .to_string(),
        );
        server_info.website_url = Some("https://puzzles.mauriulloa.com".to_string());

        let mut info = ServerConfig::default();
        info.server_info = server_info;
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.instructions = Some(
            "Makes printable chess worksheets. Call list_options, pick the level and theme \
             that fit the learner, then create_worksheet and give the person the worksheet_url \
             to open and print. Levels are bands of Lichess puzzle rating, so pick the one that \
             holds the learner's rating. When a student will use the sheet unsupervised, pass answers=none so \
             the solutions are not in front of them."
                .to_string(),
        );
        info
    }
}
