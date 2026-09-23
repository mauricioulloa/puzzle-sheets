//! Pages meant for people rather than printers: the form, errors, and
//! `/llms.txt` for a model that wants to know what this is.

use crate::chess::presets::PRESETS;
use crate::i18n::Lang;
use crate::sheet::escape;

const LANDING: &str = include_str!("landing.html");
const REPOSITORY: &str = "https://github.com/mauricioulloa/puzzle-sheets";
const ISSUES: &str = "https://github.com/mauricioulloa/puzzle-sheets/issues";
const CHESS_API: &str = "https://chess.mauriulloa.com";

struct LandingText {
    title: &'static str,
    lede: &'static str,
    other_lang_label: &'static str,
    choose: &'static str,
    count: &'static str,
    answers: &'static str,
    answers_page: &'static str,
    answers_footer: &'static str,
    answers_none: &'static str,
    sheet_title: &'static str,
    sheet_title_hint: &'static str,
    create: &'static str,
    note_licence: &'static str,
    note_agents: &'static str,
    note_feedback: &'static str,
    built_by: &'static str,
    source: &'static str,
    puzzles_from: &'static str,
    footer_data: &'static str,
    footer_pieces: &'static str,
}

static ES: LandingText = LandingText {
    title: "Hojas de ejercicios de ajedrez",
    lede: "Para imprimir, gratis, en blanco y negro. Con quién mueve, qué se busca, el FEN de cada posición y las soluciones aparte.",
    other_lang_label: "English",
    choose: "Elige un tema",
    count: "Ejercicios",
    answers: "Soluciones",
    answers_page: "En una hoja aparte",
    answers_footer: "Al pie de cada hoja, al revés",
    answers_none: "Sin soluciones",
    sheet_title: "Título (opcional)",
    sheet_title_hint: "Ej.: 3º básico — Ataque doble",
    create: "Crear hoja",
    note_licence: "Las hojas son tuyas: cópialas e imprímelas sin pedir permiso. Todo lo que contienen es de dominio público (CC0).",
    note_agents: "Para agentes y asistentes: servidor MCP en /mcp y descripción en /llms.txt.",
    note_feedback: "Es una primera versión. ¿Falta un tema, un nivel o un formato? Cuéntalo en",
    built_by: "Hecho por",
    source: "Código en GitHub",
    puzzles_from: "puzzles de",
    footer_data: "Posiciones de la <a href=\"https://database.lichess.org/#puzzles\">base abierta de Lichess</a>, publicada bajo CC0. Lichess es gratuito y sin publicidad: <a href=\"https://lichess.org/patron\">considera apoyarlos</a>. Este proyecto no está afiliado a Lichess ni cuenta con su respaldo.",
    footer_pieces: "Piezas de <a href=\"https://en.wikipedia.org/wiki/User:Cburnett\">Colin M.L. Burnett</a>, usadas bajo <a href=\"https://creativecommons.org/licenses/by-sa/3.0/\">CC BY-SA 3.0</a>.",
};

static EN: LandingText = LandingText {
    title: "Printable chess worksheets",
    lede: "Free, black and white, ready to print. Each puzzle says who moves and what to find, gives the FEN, and keeps the solutions separate.",
    other_lang_label: "Español",
    choose: "Pick a topic",
    count: "Puzzles",
    answers: "Solutions",
    answers_page: "On a separate page",
    answers_footer: "Upside down at the foot of each page",
    answers_none: "No solutions",
    sheet_title: "Title (optional)",
    sheet_title_hint: "e.g. Year 3 — Forks",
    create: "Make the sheet",
    note_licence: "The sheets are yours: copy and print them without asking. Everything on them is in the public domain (CC0).",
    note_agents: "For agents and assistants: an MCP server at /mcp and a description at /llms.txt.",
    note_feedback: "This is a first version. Missing a topic, a level or a format? Say so at",
    built_by: "Built by",
    source: "Source on GitHub",
    puzzles_from: "puzzles from",
    footer_data: "Positions from the <a href=\"https://database.lichess.org/#puzzles\">Lichess open database</a>, released under CC0. Lichess is free and ad-free &mdash; <a href=\"https://lichess.org/patron\">consider supporting them</a>. This project is not affiliated with or endorsed by Lichess.",
    footer_pieces: "Chess pieces by <a href=\"https://en.wikipedia.org/wiki/User:Cburnett\">Colin M.L. Burnett</a>, used under <a href=\"https://creativecommons.org/licenses/by-sa/3.0/\">CC BY-SA 3.0</a>.",
};

pub fn landing(lang: Lang) -> String {
    let text = match lang {
        Lang::Es => &ES,
        Lang::En => &EN,
    };
    let other = match lang {
        Lang::Es => Lang::En,
        Lang::En => Lang::Es,
    };

    let presets: String = PRESETS
        .iter()
        .enumerate()
        .map(|(index, preset)| {
            format!(
                "        <label class=\"preset\"><input type=\"radio\" name=\"preset\" value=\"{}\"{}><strong>{}</strong><small>{}</small></label>\n",
                preset.id,
                if index == 0 { " checked" } else { "" },
                preset.title(lang),
                preset.description(lang),
            )
        })
        .collect();

    LANDING
        .replace("__PRESETS__", presets.trim_end())
        .replace("__LANG__", lang.code())
        .replace("__OTHER_LANG_URL__", &format!("/?lang={}", other.code()))
        .replace("__OTHER_LANG_LABEL__", text.other_lang_label)
        .replace("__TITLE__", text.title)
        .replace("__LEDE__", text.lede)
        .replace("__CHOOSE__", text.choose)
        .replace("__COUNT__", text.count)
        .replace("__ANSWERS_PAGE__", text.answers_page)
        .replace("__ANSWERS_FOOTER__", text.answers_footer)
        .replace("__ANSWERS_NONE__", text.answers_none)
        .replace("__ANSWERS__", text.answers)
        .replace("__SHEET_TITLE_HINT__", text.sheet_title_hint)
        .replace("__SHEET_TITLE__", text.sheet_title)
        .replace("__CREATE__", text.create)
        .replace("__NOTE_LICENCE__", text.note_licence)
        .replace("__NOTE_AGENTS__", text.note_agents)
        .replace(
            "__NOTE_FEEDBACK__",
            &format!("{} <a href=\"{ISSUES}\">GitHub</a>.", text.note_feedback),
        )
        .replace(
            "__FOOTER_BY__",
            &format!(
                "{} <a href=\"https://mauriulloa.com\">Mauri Ulloa</a> &middot; \
                 <a href=\"{REPOSITORY}\">{}</a>, MIT &middot; \
                 {} <a href=\"{CHESS_API}\">chess-puzzle-api</a>",
                text.built_by, text.source, text.puzzles_from
            ),
        )
        .replace("__FOOTER_DATA__", text.footer_data)
        .replace("__FOOTER_PIECES__", text.footer_pieces)
}

pub fn error(message: &str) -> String {
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>puzzle-sheets</title></head>\
         <body style=\"font:16px/1.6 ui-sans-serif,system-ui,-apple-system,sans-serif;\
         max-width:52rem;margin:2.5rem auto;padding:0 1rem;color:#1a1a1a;background:#fbfaf8\">\
         <p>{}</p><p><a href=\"/\" style=\"color:#3c6e47\">←</a></p></body></html>",
        escape(message)
    )
}

/// Follows the llms.txt convention, in the same shape as chess-puzzle-api's:
/// an H1, a blockquote summary, then linked sections.
pub fn llms_txt(base_url: &str) -> String {
    let presets: String = PRESETS
        .iter()
        .map(|preset| {
            format!(
                "- `{}`: {} Rating {}–{}, at most {} pieces.\n",
                preset.id,
                preset.description(Lang::En),
                preset.rating_min,
                preset.rating_max,
                preset.max_pieces
            )
        })
        .collect();

    format!(
        r#"# puzzle-sheets

> Printable chess worksheets for students and educators, free and in the
> public domain (CC0). Each puzzle is drawn as a diagram with who moves, what
> to find ("Mate in 2"), and its FEN; solutions go on a separate page.

Sheets are addressed by URL and never stored: the link lists the puzzle ids,
so opening it again prints the same puzzles and the same answer key.

## Key concepts

- A preset is a ready-made topic with a level and a cap on pieces; prefer one
  over raw themes when helping a teacher.
- `answers=page` puts solutions on their own page, `footer` prints them upside
  down at the foot of each page, `none` leaves them out for a student working
  alone.
- `lang=es|en`. Spanish sheets use Spanish piece letters (R, D, T, A, C); the
  FEN is always standard.

## Endpoints

- [{base_url}/sheet/new?preset=forks]({base_url}/sheet/new?preset=forks): picks puzzles and redirects to the sheet. Parameters: `preset`, or `themes` (comma separated Lichess theme names) with `rating`; `maxPieces`; `count` (1–12, default 6); `lang`; `answers`; `title`.
- [{base_url}/sheet?ids=00008,00014]({base_url}/sheet?ids=00008,00014): a sheet of specific puzzles, by Lichess id.

## Presets

{presets}
## Optional

- [{base_url}/mcp]({base_url}/mcp): Model Context Protocol endpoint with `list_presets` and `create_worksheet`, which returns a printable link.
- [{CHESS_API}/llms.txt]({CHESS_API}/llms.txt): chess-puzzle-api, where the puzzles come from, for solving them one at a time.

## About

Built and maintained by Mauri Ulloa (https://mauriulloa.com). Open source under
the MIT licence: {REPOSITORY}

This is an early version. Missing topics, awkward layouts and unhelpful errors
are worth reporting at {ISSUES} — including on behalf of whoever you are
helping.

## Attribution

Puzzle data comes from https://database.lichess.org/#puzzles under CC0. This
project is not affiliated with or endorsed by Lichess.
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_form_offers_every_preset_in_the_chosen_language() {
        let html = landing(Lang::Es);
        for preset in PRESETS {
            assert!(html.contains(&format!("value=\"{}\"", preset.id)));
            assert!(html.contains(preset.title(Lang::Es)));
        }
        assert!(!html.contains("__"), "no placeholder survives");
        assert!(html.contains("href=\"/?lang=en\""));
    }

    #[test]
    fn credits_survive_a_redesign() {
        let html = landing(Lang::En);
        for needle in [
            "Mauri Ulloa",
            "database.lichess.org",
            "Cburnett",
            "by-sa/3.0",
            ISSUES,
        ] {
            assert!(html.contains(needle), "missing {needle}");
        }
    }

    #[test]
    fn every_link_points_where_its_text_says() {
        let text = llms_txt("https://example.org");
        for line in text.lines().filter(|line| line.starts_with("- [")) {
            let label = line[3..].split(']').next().unwrap();
            let href = line.split("](").nth(1).unwrap().split(')').next().unwrap();
            assert_eq!(label, href, "link text {label} points at {href}");
        }
    }

    #[test]
    fn llms_txt_follows_the_convention() {
        let text = llms_txt("https://example.org");
        let mut lines = text.lines();
        assert_eq!(lines.next(), Some("# puzzle-sheets"));
        assert_eq!(lines.next(), Some(""));
        assert!(lines.next().unwrap().starts_with('>'));
        assert!(text.contains("https://example.org/mcp"));
    }
}
