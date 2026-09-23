//! What a sheet can be asked for: a difficulty and a theme.
//!
//! The rating is the one truth: every puzzle carries the rating Lichess gives
//! it, and a level is nothing but a name for a band of it. The bands are
//! contiguous and never overlap, so a rating belongs to exactly one level,
//! and chess-puzzle-api's trainer uses the same five. Lichess itself has no
//! fixed levels; its difficulty is relative to each player's own rating.
//!
//! Theme names are Lichess's own, in both languages.

use crate::i18n::Lang;

/// The highest rating chess-puzzle-api accepts.
const RATING_CEILING: u32 = 4000;

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
}

impl Level {
    /// The band as a reader sees it: "under 1000", "1000–1399", "2200+".
    pub fn range(&self, lang: Lang) -> String {
        match (self.rating_min, self.rating_max) {
            (0, max) => format!("{} {}", lang.text().under, max + 1),
            (min, RATING_CEILING) => format!("{min}+"),
            (min, max) => format!("{min}–{max}"),
        }
    }
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
        rating_min: 0,
        rating_max: 999,
    },
    Level {
        id: "novice",
        label: Label(["Inicial", "Novice"]),
        rating_min: 1000,
        rating_max: 1399,
    },
    Level {
        id: "intermediate",
        label: Label(["Intermedio", "Intermediate"]),
        rating_min: 1400,
        rating_max: 1799,
    },
    Level {
        id: "advanced",
        label: Label(["Avanzado", "Advanced"]),
        rating_min: 1800,
        rating_max: 2199,
    },
    Level {
        id: "expert",
        label: Label(["Experto", "Expert"]),
        rating_min: 2200,
        rating_max: RATING_CEILING,
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
        label: Label(["Pincho", "Skewer"]),
    },
    Theme {
        id: "discoveredAttack",
        label: Label(["Ataque a la descubierta", "Discovered attack"]),
    },
    Theme {
        id: "xRayAttack",
        label: Label(["Ataque por rayos X", "X-Ray attack"]),
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
        label: Label(["Ataque en el flanco de rey", "Kingside attack"]),
    },
    Theme {
        id: "queensideAttack",
        label: Label(["Ataque en el flanco de dama", "Queenside attack"]),
    },
    Theme {
        id: "defensiveMove",
        label: Label(["Movimiento defensivo", "Defensive move"]),
    },
    Theme {
        id: "equality",
        label: Label(["Igualdad", "Equality"]),
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
    fn every_rating_belongs_to_exactly_one_level() {
        assert_eq!(LEVELS[0].rating_min, 0);
        assert_eq!(LEVELS[LEVELS.len() - 1].rating_max, RATING_CEILING);
        for pair in LEVELS.windows(2) {
            assert_eq!(
                pair[1].rating_min,
                pair[0].rating_max + 1,
                "no gap, no overlap"
            );
        }
        assert!(level(DEFAULT_LEVEL).is_some());
    }

    #[test]
    fn a_band_reads_as_numbers() {
        let ranges: Vec<String> = LEVELS.iter().map(|level| level.range(Lang::En)).collect();
        assert_eq!(
            ranges,
            ["under 1000", "1000–1399", "1400–1799", "1800–2199", "2200+"]
        );
        assert_eq!(LEVELS[0].range(Lang::Es), "menos de 1000");
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
