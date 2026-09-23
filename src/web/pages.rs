//! Pages meant for people rather than printers: the form, errors, and
//! `/llms.txt` for a model that wants to know what this is.

use crate::chess::presets::PRESETS;
use crate::i18n::Lang;
use crate::sheet::escape;

const LANDING: &str = include_str!("landing.html");
const ISSUES: &str = "https://github.com/mauricioulloa/puzzle-sheets/issues";

struct LandingText {
    title: &'static str,
    tagline: &'static str,
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
}

static ES: LandingText = LandingText {
    title: "Hojas de ejercicios de ajedrez",
    tagline: "Para imprimir, gratis, en blanco y negro. Con quién mueve, qué se busca, el FEN de cada posición y las soluciones aparte.",
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
    note_licence: "Las posiciones vienen de la base de puzzles de Lichess, de dominio público (CC0). Las hojas son tuyas: cópialas e imprímelas sin pedir permiso.",
    note_agents: "Para agentes y asistentes: servidor MCP en /mcp y descripción en /llms.txt.",
    note_feedback: "Es una primera versión. ¿Falta un tema, un nivel o un formato? Cuéntalo en",
};

static EN: LandingText = LandingText {
    title: "Printable chess worksheets",
    tagline: "Free, black and white, ready to print. Each puzzle says who moves and what to find, gives the FEN, and keeps the solutions separate.",
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
    note_licence: "Positions come from the Lichess puzzle database, public domain (CC0). The sheets are yours: copy and print them without asking.",
    note_agents: "For agents and assistants: an MCP server at /mcp and a description at /llms.txt.",
    note_feedback: "This is a first version. Missing a topic, a level or a format? Say so at",
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
        .replace("__TAGLINE__", text.tagline)
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
            "__NOTE_CREDITS__",
            "<a href=\"https://mauriulloa.com\">Mauri Ulloa</a> · \
             Puzzles: <a href=\"https://database.lichess.org/#puzzles\">Lichess</a> (CC0), not affiliated with Lichess · \
             Pieces: <a href=\"https://en.wikipedia.org/wiki/User:Cburnett\">Colin M.L. Burnett</a>, \
             <a href=\"https://creativecommons.org/licenses/by-sa/3.0/\">CC BY-SA 3.0</a>",
        )
}

pub fn error(message: &str) -> String {
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>puzzle-sheets</title></head>\
         <body style=\"font:16px/1.5 system-ui,sans-serif;max-width:640px;margin:48px auto;padding:0 16px\">\
         <p>{}</p><p><a href=\"/\">←</a></p></body></html>",
        escape(message)
    )
}

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
so opening it again prints the same puzzles and the same answer key. Spanish
and English.

## Making a sheet

- [{base_url}/sheet/new?preset=forks]({base_url}/sheet/new?preset=forks): picks puzzles and redirects to the sheet. Parameters: `preset`, or `themes` (comma separated Lichess theme names) with `rating`; `maxPieces`; `count` (1–12, default 6); `lang=es|en`; `answers=page|footer|none`; `title`.
- [{base_url}/sheet?ids=00008,00014]({base_url}/sheet?ids=00008,00014): a sheet of specific puzzles, by Lichess id.

## Presets

{presets}
## MCP

[{base_url}/mcp]({base_url}/mcp) serves `list_presets` and `create_worksheet`,
which returns a printable link.

## About

Built by Mauri Ulloa (https://mauriulloa.com). Puzzles come from
chess-puzzle-api (https://chess.mauriulloa.com), which serves the Lichess
puzzle database (CC0). Not affiliated with Lichess.

This is a first version. Report what is missing at {ISSUES}.
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
    fn llms_txt_follows_the_convention() {
        let text = llms_txt("https://example.org");
        let mut lines = text.lines();
        assert_eq!(lines.next(), Some("# puzzle-sheets"));
        assert_eq!(lines.next(), Some(""));
        assert!(lines.next().unwrap().starts_with('>'));
        assert!(text.contains("https://example.org/mcp"));
    }
}
