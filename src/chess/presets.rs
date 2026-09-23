//! Teaching presets. A teacher picks "Mate in one", not an Elo band and a
//! theme id.
//!
//! The piece caps carry the finding that shaped this project: a low rating
//! does not mean a simple position. Mate-in-ones under 1000 have a median of
//! nineteen pieces on the board, and a beginner has to scan every one.

use crate::i18n::Lang;

pub struct Preset {
    pub id: &'static str,
    /// Spanish, then English.
    title: [&'static str; 2],
    description: [&'static str; 2],
    /// Lichess theme names; a puzzle needs any one of them.
    pub themes: &'static [&'static str],
    pub rating_min: u32,
    pub rating_max: u32,
    pub max_pieces: u32,
}

impl Preset {
    pub fn title(&self, lang: Lang) -> &'static str {
        pick(self.title, lang)
    }

    pub fn description(&self, lang: Lang) -> &'static str {
        pick(self.description, lang)
    }
}

fn pick([es, en]: [&'static str; 2], lang: Lang) -> &'static str {
    match lang {
        Lang::Es => es,
        Lang::En => en,
    }
}

pub fn find(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|preset| preset.id == id)
}

/// Ordered roughly by the order a beginner meets them.
pub const PRESETS: &[Preset] = &[
    Preset {
        id: "mate-in-1",
        title: ["Mate en 1", "Mate in 1"],
        description: [
            "Primeros mates, en tableros con pocas piezas.",
            "First checkmates, on boards with few pieces.",
        ],
        themes: &["mateIn1"],
        rating_min: 400,
        rating_max: 1000,
        max_pieces: 12,
    },
    Preset {
        id: "hanging-pieces",
        title: ["Piezas colgadas", "Hanging pieces"],
        description: [
            "Encuentra la pieza que quedó sin defensa y captúrala.",
            "Spot the undefended piece and take it.",
        ],
        themes: &["hangingPiece"],
        rating_min: 500,
        rating_max: 1100,
        max_pieces: 16,
    },
    Preset {
        id: "forks",
        title: ["Ataque doble", "Forks"],
        description: [
            "Una pieza ataca a dos a la vez. El mejor primer tema táctico.",
            "One piece attacks two at once. The best first tactic to learn.",
        ],
        themes: &["fork"],
        rating_min: 600,
        rating_max: 1200,
        max_pieces: 14,
    },
    Preset {
        id: "back-rank",
        title: ["Mate del pasillo", "Back-rank mate"],
        description: [
            "El rey queda encerrado detrás de sus propios peones.",
            "The king is trapped behind its own pawns.",
        ],
        themes: &["backRankMate"],
        rating_min: 600,
        rating_max: 1200,
        max_pieces: 16,
    },
    Preset {
        id: "mate-in-2",
        title: ["Mate en 2", "Mate in 2"],
        description: [
            "Dos jugadas para dar mate: hay que ver la respuesta del rival.",
            "Two moves to mate: the opponent's reply has to be seen.",
        ],
        themes: &["mateIn2"],
        rating_min: 800,
        rating_max: 1300,
        max_pieces: 14,
    },
    Preset {
        id: "pins-and-skewers",
        title: ["Clavadas y enfiladas", "Pins and skewers"],
        description: [
            "Ataques en línea contra dos piezas.",
            "Attacks along a line through two pieces.",
        ],
        themes: &["pin", "skewer"],
        rating_min: 800,
        rating_max: 1400,
        max_pieces: 16,
    },
    Preset {
        id: "pawn-endgames",
        title: ["Finales de peones", "Pawn endgames"],
        description: [
            "Reyes y peones: coronar o impedir la coronación.",
            "Kings and pawns: promote, or stop the promotion.",
        ],
        themes: &["pawnEndgame"],
        rating_min: 700,
        rating_max: 1400,
        max_pieces: 8,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_ranges_sane() {
        for (index, preset) in PRESETS.iter().enumerate() {
            assert!(
                PRESETS[..index].iter().all(|other| other.id != preset.id),
                "duplicate id {}",
                preset.id
            );
            assert!(preset.rating_min < preset.rating_max, "{}", preset.id);
            assert!((2..=32).contains(&preset.max_pieces), "{}", preset.id);
            assert!(!preset.themes.is_empty(), "{}", preset.id);
        }
    }

    #[test]
    fn every_preset_speaks_both_languages() {
        for preset in PRESETS {
            for lang in [Lang::Es, Lang::En] {
                assert!(!preset.title(lang).is_empty());
                assert!(!preset.description(lang).is_empty());
            }
        }
    }
}
