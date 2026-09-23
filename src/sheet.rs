//! A printable worksheet, independent of the kind of puzzle on it.
//!
//! Every puzzle type reduces to the same few things: a diagram, a line telling
//! the student what to do, an optional detail for replaying it elsewhere, and
//! the solution. The sheet lays those out; it never knows it is chess.
//!
//! The layout is deliberately bare, like a printed puzzle book: a title, a
//! number over each diagram, and nothing else that costs ink.

use crate::i18n::Lang;

/// Six to a page: two columns of three leaves each diagram large enough to
/// read.
pub const PER_PAGE: usize = 6;
pub const MAX_ITEMS: usize = 12;

pub struct Item {
    /// Inline SVG.
    pub diagram: String,
    pub prompt: String,
    /// Small print under the prompt, such as a FEN to load into a program.
    pub detail: Option<String>,
    pub solution: String,
}

/// Where the solutions go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Answers {
    /// Upside down at the foot of each page, as puzzle books do.
    #[default]
    Footer,
    /// On a page of their own right after each page of puzzles, so printing
    /// double-sided puts them on the back.
    Page,
    None,
}

impl Answers {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "footer" => Some(Self::Footer),
            "page" => Some(Self::Page),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Footer => "footer",
            Self::Page => "page",
            Self::None => "none",
        }
    }
}

pub struct Sheet {
    pub title: String,
    /// A line under the title, such as the difficulty and theme.
    pub subtitle: Option<String>,
    pub items: Vec<Item>,
    pub lang: Lang,
    pub answers: Answers,
    /// Markup every page needs once, such as SVG symbol definitions.
    pub defs: &'static str,
    /// Where "another sheet" leads.
    pub again_url: String,
}

const STYLE: &str = include_str!("web/sheet.css");

pub fn render(sheet: &Sheet) -> String {
    let text = sheet.lang.text();
    let title = escape(&sheet.title);
    let subtitle = sheet
        .subtitle
        .as_deref()
        .map(|subtitle| format!("<p class=\"subtitle\">{}</p>", escape(subtitle)))
        .unwrap_or_default();
    let pages: Vec<&[Item]> = sheet.items.chunks(PER_PAGE).collect();
    let page_count = pages.len();

    let mut body = String::new();
    for (page_index, items) in pages.iter().enumerate() {
        let first_number = page_index * PER_PAGE + 1;
        body.push_str("<section class=\"page\">");
        body.push_str(&format!("<header><h1>{title}</h1>{subtitle}</header>"));
        body.push_str("<div class=\"grid\">");
        for (offset, item) in items.iter().enumerate() {
            body.push_str(&render_item(first_number + offset, item));
        }
        body.push_str("</div>");
        if sheet.answers == Answers::Footer {
            body.push_str(&format!(
                "<footer class=\"upside-down\">{}</footer>",
                solution_list(first_number, items, false)
            ));
        }
        body.push_str(&format!(
            "<p class=\"page-number\">{} / {page_count}</p><p class=\"credit\">{} · puzzles.mauriulloa.com</p>",
            page_index + 1,
            text.licence
        ));
        body.push_str("</section>");
        if sheet.answers == Answers::Page {
            body.push_str(&format!(
                "<section class=\"page answers\"><header><h1>{title}</h1><p class=\"subtitle\">{}</p></header>{}</section>",
                text.solutions,
                solution_list(first_number, items, true)
            ));
        }
    }

    format!(
        r#"<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<style>{STYLE}</style>
</head>
<body>
<nav class="toolbar"><button onclick="window.print()">{print}</button><a href="{again}">{again_label}</a></nav>
{defs}
{body}
</body>
</html>
"#,
        lang = sheet.lang.code(),
        print = text.print,
        again = escape(&sheet.again_url),
        again_label = text.new_sheet,
        defs = sheet.defs,
    )
}

fn render_item(number: usize, item: &Item) -> String {
    let detail = item
        .detail
        .as_deref()
        .map(|detail| format!("<p class=\"detail\">{}</p>", escape(detail)))
        .unwrap_or_default();
    format!(
        "<article class=\"item\"><h2>{number}</h2>{diagram}<p class=\"prompt\">{prompt}</p>{detail}</article>",
        diagram = item.diagram,
        prompt = escape(&item.prompt),
    )
}

/// A page of its own repeats each prompt so it reads alone; the footer sits
/// under the prompts already, so it keeps to the moves.
fn solution_list(first_number: usize, items: &[Item], with_prompts: bool) -> String {
    let entries: String = items
        .iter()
        .enumerate()
        .map(|(offset, item)| {
            let prompt = if with_prompts {
                format!("{}: ", escape(&item.prompt))
            } else {
                String::new()
            };
            // The number is written out rather than left to the list marker:
            // "6. 1. c4+" reads as two move numbers.
            format!(
                "<li><strong>{}</strong> — {prompt}{}</li>",
                first_number + offset,
                escape(&item.solution)
            )
        })
        .collect();
    format!("<ol class=\"solutions\">{entries}</ol>")
}

pub fn escape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(n: usize) -> Item {
        Item {
            diagram: "<svg></svg>".to_string(),
            prompt: format!("Prompt {n}"),
            detail: Some(format!("detail {n}")),
            solution: format!("SOLUTION-{n}"),
        }
    }

    fn sheet(count: usize, answers: Answers) -> Sheet {
        Sheet {
            title: "Forks <b>".to_string(),
            subtitle: Some("Difficulty: Novice".to_string()),
            items: (1..=count).map(item).collect(),
            lang: Lang::En,
            answers,
            defs: "",
            again_url: "/".to_string(),
        }
    }

    #[test]
    fn solutions_sit_upside_down_below_their_own_puzzles() {
        let html = render(&sheet(12, Answers::Footer));
        assert_eq!(html.matches("class=\"upside-down\"").count(), 2);
        let second_page = html.find("<strong>7</strong>").expect("page two solutions");
        assert!(
            html.find("detail 12").unwrap() < second_page,
            "each page's solutions follow its own puzzles"
        );
    }

    #[test]
    fn a_solutions_page_follows_each_page_of_puzzles() {
        let html = render(&sheet(12, Answers::Page));
        let pages: Vec<usize> = html
            .match_indices("<section class=\"page")
            .map(|(at, _)| at)
            .collect();
        assert_eq!(pages.len(), 4, "puzzles, solutions, puzzles, solutions");
        let at = |needle: &str| html.find(needle).expect(needle);
        assert!(at("detail 6") < pages[1] && pages[1] < at("SOLUTION-1"));
        assert!(at("SOLUTION-6") < pages[2] && pages[2] < at("detail 7"));
        assert!(at("detail 12") < pages[3] && pages[3] < at("SOLUTION-7"));
        assert!(!html.contains("class=\"upside-down\""));
        assert!(
            html.contains("Prompt 1: SOLUTION-1"),
            "the page reads on its own"
        );
    }

    #[test]
    fn a_sheet_without_answers_carries_none() {
        let html = render(&sheet(6, Answers::None));
        assert!(!html.contains("SOLUTION-"));
    }

    #[test]
    fn twelve_puzzles_make_two_numbered_pages() {
        let html = render(&sheet(12, Answers::None));
        assert_eq!(html.matches("<section class=\"page\">").count(), 2);
        assert!(html.contains("2 / 2"));
        assert!(html.contains("<h2>12</h2>"));
    }

    #[test]
    fn the_subtitle_is_printed_under_the_title() {
        let html = render(&sheet(1, Answers::None));
        assert!(html.contains("<p class=\"subtitle\">Difficulty: Novice</p>"));
    }

    #[test]
    fn a_title_cannot_inject_markup() {
        let html = render(&sheet(1, Answers::None));
        assert!(html.contains("Forks &lt;b&gt;"));
        assert!(!html.contains("Forks <b>"));
    }
}
