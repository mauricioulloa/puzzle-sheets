//! A Model Context Protocol server, so an assistant helping a teacher, a
//! parent or a student can hand them a printable sheet instead of describing
//! positions in chat.

use crate::chess::presets::PRESETS;
use crate::i18n::Lang;
use crate::sheet::Answers;
use crate::web::SharedState;
use crate::worksheet::{self, WorksheetError};
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{ErrorData, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, schemars, tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPresetsArgs {
    /// `es` or `en`. Defaults to `es`.
    pub lang: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PresetInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub themes: Vec<String>,
    pub rating_min: u32,
    pub rating_max: u32,
    pub max_pieces: u32,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Presets {
    pub presets: Vec<PresetInfo>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateWorksheetArgs {
    /// A preset id from `list_presets`, e.g. `forks` or `mate-in-1`. The
    /// simplest choice for a teacher.
    pub preset: Option<String>,
    /// Lichess theme names, instead of or on top of a preset, e.g. `pin`.
    pub themes: Option<Vec<String>>,
    /// Target Elo rating; puzzles come from 150 either side. Roughly: 600 is
    /// a young beginner, 1000 a school player, 1500 a club player.
    pub rating: Option<u32>,
    /// At most this many pieces on the board. 8 to 14 keeps positions simple
    /// enough for children.
    pub max_pieces: Option<u32>,
    /// How many puzzles, 1 to 12. Six fill a page. Defaults to 6.
    pub count: Option<usize>,
    /// `es` or `en`. Defaults to `es`.
    pub lang: Option<String>,
    /// `page` puts solutions on a separate page (default), `footer` prints
    /// them upside down at the foot of each page, `none` leaves them out —
    /// use `none` for a sheet a student should not see the answers to.
    pub answers: Option<String>,
    /// Heading printed on the sheet, e.g. a class name.
    pub title: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Worksheet {
    /// Open and print this. It is permanent: it always shows these puzzles.
    pub worksheet_url: String,
    pub title: String,
    pub puzzle_ids: Vec<String>,
}

fn invalid(message: impl Into<String>) -> ErrorData {
    ErrorData::invalid_params(message.into(), None)
}

fn parse_lang(raw: Option<&str>) -> Result<Lang, ErrorData> {
    match raw {
        None => Ok(Lang::default()),
        Some(value) => Lang::parse(value).ok_or_else(|| invalid("lang must be es or en")),
    }
}

impl From<WorksheetError> for ErrorData {
    fn from(err: WorksheetError) -> Self {
        match err {
            WorksheetError::Invalid(message) => invalid(message),
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
        name = "list_presets",
        description = "List the teaching presets for chess worksheets — ready-made topics such \
                       as mate in one or forks, each with a sensible level and a cap on pieces \
                       so positions stay simple. Call this before create_worksheet."
    )]
    async fn list_presets(
        &self,
        Parameters(args): Parameters<ListPresetsArgs>,
    ) -> Result<Json<Presets>, ErrorData> {
        let lang = parse_lang(args.lang.as_deref())?;
        let presets = PRESETS
            .iter()
            .map(|preset| PresetInfo {
                id: preset.id.to_string(),
                title: preset.title(lang).to_string(),
                description: preset.description(lang).to_string(),
                themes: preset
                    .themes
                    .iter()
                    .map(|theme| theme.to_string())
                    .collect(),
                rating_min: preset.rating_min,
                rating_max: preset.rating_max,
                max_pieces: preset.max_pieces,
            })
            .collect();
        Ok(Json(Presets { presets }))
    }

    #[tool(
        name = "create_worksheet",
        description = "Make a printable chess worksheet and return its link. Each puzzle shows \
                       who moves, what to find (e.g. 'Mate in 2') and its FEN; solutions go on \
                       a separate page unless asked otherwise. Use this when a teacher, parent \
                       or student wants puzzles on paper or a sheet to practise with."
    )]
    async fn create_worksheet(
        &self,
        Parameters(args): Parameters<CreateWorksheetArgs>,
    ) -> Result<Json<Worksheet>, ErrorData> {
        let answers = match args.answers.as_deref() {
            None => Answers::default(),
            Some(value) => Answers::parse(value)
                .ok_or_else(|| invalid("answers must be page, footer or none"))?,
        };
        let request = worksheet::Request {
            preset: args.preset,
            themes: args.themes.unwrap_or_default(),
            rating: args.rating,
            max_pieces: args.max_pieces,
            count: args.count,
            lang: parse_lang(args.lang.as_deref())?,
            answers,
            title: args.title,
        };
        let created = worksheet::create(&self.state.api, request).await?;
        Ok(Json(Worksheet {
            worksheet_url: format!("{}{}", self.state.public_url, created.path),
            title: created.title,
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
            "Makes printable chess worksheets. Call list_presets, pick the one that fits the \
             learner, then create_worksheet and give the person the worksheet_url to open and \
             print. For young beginners prefer presets with few pieces over a low rating alone. \
             When a student will use the sheet unsupervised, pass answers=none so the solutions \
             are not in front of them."
                .to_string(),
        );
        info
    }
}
