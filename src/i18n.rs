//! Spanish and English. Sheets carry little text, so every fixed string lives
//! here rather than in a translation framework. Messages with a value in them
//! live beside the error they describe, in `worksheet::Invalid`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    Es,
    En,
}

impl Lang {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "es" => Some(Self::Es),
            "en" => Some(Self::En),
            _ => None,
        }
    }

    /// Spanish for anyone whose browser prefers it, English otherwise.
    pub fn from_accept_language(header: Option<&str>) -> Self {
        let prefers_spanish = header.is_some_and(|value| {
            value
                .split(',')
                .next()
                .is_some_and(|first| first.trim().to_ascii_lowercase().starts_with("es"))
        });
        if prefers_spanish { Self::Es } else { Self::En }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Es => "es",
            Self::En => "en",
        }
    }

    pub fn text(self) -> &'static Text {
        match self {
            Self::Es => &ES,
            Self::En => &EN,
        }
    }
}

pub struct Text {
    pub default_title: &'static str,
    pub difficulty: &'static str,
    pub theme: &'static str,
    pub under: &'static str,
    pub solutions: &'static str,
    pub print: &'static str,
    pub new_sheet: &'static str,
    pub goal_mate: &'static str,
    pub goal_mate_in: &'static str,
    pub goal_crushing: &'static str,
    pub goal_advantage: &'static str,
    pub goal_equality: &'static str,
    pub goal_best_move: &'static str,
    pub licence: &'static str,
    pub no_puzzles: &'static str,
    pub service_down: &'static str,
    pub service_busy: &'static str,
    pub seconds: &'static str,
    pub back: &'static str,
    pub white_to_move: &'static str,
    pub black_to_move: &'static str,
}

static ES: Text = Text {
    default_title: "Ejercicios de ajedrez",
    difficulty: "Dificultad",
    theme: "Tema",
    under: "menos de",
    solutions: "Soluciones",
    print: "Imprimir",
    new_sheet: "Otra hoja",
    goal_mate: "Da mate",
    goal_mate_in: "Mate en",
    goal_crushing: "Gana con ventaja decisiva",
    goal_advantage: "Consigue ventaja",
    goal_equality: "Salva la partida",
    goal_best_move: "Encuentra la mejor jugada",
    licence: "Libre para copiar e imprimir (CC0).",
    no_puzzles: "No hay suficientes ejercicios de ese tema en esa dificultad. Prueba con otra dificultad o con cualquier tema.",
    service_down: "El servicio de ejercicios no respondió. Inténtalo de nuevo en un momento.",
    service_busy: "El servicio de ejercicios está ocupado. Inténtalo de nuevo en",
    seconds: "segundos",
    back: "Volver al formulario",
    white_to_move: "Juegan las blancas",
    black_to_move: "Juegan las negras",
};

static EN: Text = Text {
    default_title: "Chess exercises",
    difficulty: "Difficulty",
    theme: "Theme",
    under: "under",
    solutions: "Solutions",
    print: "Print",
    new_sheet: "Another sheet",
    goal_mate: "Checkmate",
    goal_mate_in: "Mate in",
    goal_crushing: "Win decisively",
    goal_advantage: "Gain the advantage",
    goal_equality: "Hold the draw",
    goal_best_move: "Find the best move",
    licence: "Free to copy and print (CC0).",
    no_puzzles: "There are not enough puzzles on that theme at that difficulty. Try another difficulty, or any theme.",
    service_down: "The puzzle service did not answer. Try again in a moment.",
    service_busy: "The puzzle service is busy. Try again in",
    seconds: "seconds",
    back: "Back to the form",
    white_to_move: "White to move",
    black_to_move: "Black to move",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spanish_only_when_the_browser_prefers_it() {
        assert_eq!(Lang::from_accept_language(Some("es-CL,es;q=0.9")), Lang::Es);
        assert_eq!(Lang::from_accept_language(Some("en-US,es;q=0.5")), Lang::En);
        assert_eq!(Lang::from_accept_language(None), Lang::En);
    }
}
