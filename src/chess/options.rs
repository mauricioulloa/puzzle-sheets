//! What a sheet can be asked for: a difficulty and a theme.
//!
//! The two easiest levels cap the pieces on the board, because a low rating
//! does not mean a simple position: mate-in-ones under 1000 have a median of
//! nineteen pieces, and a beginner has to scan every one.

use crate::i18n::Lang;

/// A label in Spanish, then English.
pub struct Label([&'static str; 2]);

impl Label {
    pub fn get(&self, lang: Lang) -> &'static str {
        let Label([es, en]) = self;
        match lang {
            Lang::Es => es,
            Lang::En => en,
        }
    }
}

pub struct Level {
    pub id: &'static str,
    pub label: Label,
    pub rating_min: u32,
    pub rating_max: u32,
    pub max_pieces: Option<u32>,
}

pub struct Theme {
    /// The Lichess theme name, as chess-puzzle-api takes it.
    pub id: &'static str,
    pub label: Label,
}

pub const DEFAULT_LEVEL: &str = "novice";

pub const LEVELS: &[Level] = &[
    Level {
        id: "beginner",
        label: Label(["Principiante", "Beginner"]),
        rating_min: 400,
        rating_max: 1000,
        max_pieces: Some(12),
    },
    Level {
        id: "novice",
        label: Label(["Inicial", "Novice"]),
        rating_min: 900,
        rating_max: 1300,
        max_pieces: Some(16),
    },
    Level {
        id: "intermediate",
        label: Label(["Intermedio", "Intermediate"]),
        rating_min: 1300,
        rating_max: 1700,
        max_pieces: None,
    },
    Level {
        id: "advanced",
        label: Label(["Avanzado", "Advanced"]),
        rating_min: 1700,
        rating_max: 2100,
        max_pieces: None,
    },
    Level {
        id: "expert",
        label: Label(["Experto", "Expert"]),
        rating_min: 2100,
        rating_max: 2800,
        max_pieces: None,
    },
];

pub const THEMES: &[Theme] = &[
    Theme {
        id: "mateIn1",
        label: Label(["Mate en 1", "Mate in 1"]),
    },
    Theme {
        id: "mateIn2",
        label: Label(["Mate en 2", "Mate in 2"]),
    },
    Theme {
        id: "mateIn3",
        label: Label(["Mate en 3", "Mate in 3"]),
    },
    Theme {
        id: "mateIn4",
        label: Label(["Mate en 4", "Mate in 4"]),
    },
    Theme {
        id: "mateIn5",
        label: Label(["Mate en 5 o más", "Mate in 5 or more"]),
    },
    Theme {
        id: "sacrifice",
        label: Label(["Sacrificio", "Sacrifice"]),
    },
    Theme {
        id: "pawnEndgame",
        label: Label(["Final de peones", "Pawn endgame"]),
    },
    Theme {
        id: "rookEndgame",
        label: Label(["Final de torres", "Rook endgame"]),
    },
    Theme {
        id: "bishopEndgame",
        label: Label(["Final de alfiles", "Bishop endgame"]),
    },
    Theme {
        id: "queenEndgame",
        label: Label(["Final de damas", "Queen endgame"]),
    },
    Theme {
        id: "fork",
        label: Label(["Ataque doble", "Fork"]),
    },
    Theme {
        id: "pin",
        label: Label(["Clavada", "Pin"]),
    },
    Theme {
        id: "skewer",
        label: Label(["Enfilada", "Skewer"]),
    },
    Theme {
        id: "discoveredAttack",
        label: Label(["Ataque a la descubierta", "Discovered attack"]),
    },
    Theme {
        id: "xRayAttack",
        label: Label(["Rayos X", "X-ray"]),
    },
    Theme {
        id: "deflection",
        label: Label(["Desviación", "Deflection"]),
    },
    Theme {
        id: "attraction",
        label: Label(["Atracción", "Attraction"]),
    },
    Theme {
        id: "clearance",
        label: Label(["Despeje", "Clearance"]),
    },
    Theme {
        id: "interference",
        label: Label(["Interferencia", "Interference"]),
    },
    Theme {
        id: "intermezzo",
        label: Label(["Jugada intermedia", "Intermezzo"]),
    },
    Theme {
        id: "kingsideAttack",
        label: Label(["Ataque al flanco de rey", "Kingside attack"]),
    },
    Theme {
        id: "queensideAttack",
        label: Label(["Ataque al flanco de dama", "Queenside attack"]),
    },
    Theme {
        id: "defensiveMove",
        label: Label(["Jugada defensiva", "Defensive move"]),
    },
    Theme {
        id: "equality",
        label: Label(["Igualar", "Equality"]),
    },
    Theme {
        id: "zugzwang",
        label: Label(["Zugzwang", "Zugzwang"]),
    },
];

pub fn level(id: &str) -> Option<&'static Level> {
    LEVELS.iter().find(|level| level.id == id)
}

pub fn theme(id: &str) -> Option<&'static Theme> {
    THEMES.iter().find(|theme| theme.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_climb_without_gaps() {
        for pair in LEVELS.windows(2) {
            assert!(pair[0].rating_min < pair[1].rating_min);
            assert!(
                pair[1].rating_min <= pair[0].rating_max,
                "no band is skipped"
            );
        }
        assert!(level(DEFAULT_LEVEL).is_some());
    }

    #[test]
    fn ids_are_unique_and_labels_bilingual() {
        for (index, theme) in THEMES.iter().enumerate() {
            assert!(
                THEMES[..index].iter().all(|other| other.id != theme.id),
                "{}",
                theme.id
            );
            assert!(!theme.label.get(Lang::Es).is_empty() && !theme.label.get(Lang::En).is_empty());
        }
    }
}
