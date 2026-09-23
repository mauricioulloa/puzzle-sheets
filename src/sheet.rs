//! A printable worksheet, independent of the kind of puzzle on it.
//!
//! Every puzzle type reduces to the same few things: a diagram, a line telling
//! the student what to do, an optional detail for replaying it elsewhere, and
//! the solution. The sheet lays those out; it never knows it is chess.

use crate::i18n::Lang;

/// Six to a page: two columns of three leaves each diagram large enough to
/// read and room under it to write.
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
    /// On their own page, so a teacher can keep them.
    #[default]
    Page,
    /// Upside down at the foot of each page, as puzzle books do.
    Footer,
    None,
}

impl Answers {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "page" => Some(Self::Page),
            "footer" => Some(Self::Footer),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Footer => "footer",
            Self::None => "none",
        }
    }
}

pub struct Sheet {
    pub title: String,
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
    let pages: Vec<&[Item]> = sheet.items.chunks(PER_PAGE).collect();
    let page_count = pages.len();

    let mut body = String::new();
    for (page_index, items) in pages.iter().enumerate() {
        let first_number = page_index * PER_PAGE + 1;
        body.push_str("<section class=\"page\">");
        body.push_str(&format!(
            "<header><h1>{title}</h1><div class=\"fields\"><span>{}: </span><span>{}: </span></div></header>",
            text.name, text.date
        ));
        body.push_str("<div class=\"grid\">");
        for (offset, item) in items.iter().enumerate() {
            body.push_str(&render_item(first_number + offset, item, text.answer));
        }
        body.push_str("</div>");
        if sheet.answers == Answers::Footer {
            body.push_str(&format!(
                "<footer class=\"upside-down\">{}</footer>",
                solution_list(first_number, items, false)
            ));
        }
        body.push_str(&format!(
            "<p class=\"credit\">{} · puzzles.mauriulloa.com{}</p>",
            text.licence,
            if page_count > 1 {
                format!(" · {}/{page_count}", page_index + 1)
            } else {
                String::new()
            }
        ));
        body.push_str("</section>");
    }

    if sheet.answers == Answers::Page {
        body.push_str(&format!(
            "<section class=\"page key\"><header><h1>{title} — {}</h1></header>{}</section>",
            text.answers,
            solution_list(1, &sheet.items, true)
        ));
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

fn render_item(number: usize, item: &Item, answer_label: &str) -> String {
    let detail = item
        .detail
        .as_deref()
        .map(|detail| format!("<p class=\"detail\">{}</p>", escape(detail)))
        .unwrap_or_default();
    format!(
        "<article class=\"item\"><div class=\"number\">{number}</div>{diagram}<p class=\"prompt\">{prompt}</p>{detail}<p class=\"answer\">{answer_label}: </p></article>",
        diagram = item.diagram,
        prompt = escape(&item.prompt),
    )
}

/// The answer page repeats each prompt so it reads on its own; the footer
/// sits under the prompts already, so it keeps to the moves.
fn solution_list(first_number: usize, items: &[Item], with_prompts: bool) -> String {
    let entries: String = items
        .iter()
        .enumerate()
        .map(|(offset, item)| {
            let prompt = if with_prompts {
                format!("<span class=\"prompt\">{}</span> ", escape(&item.prompt))
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
            items: (1..=count).map(item).collect(),
            lang: Lang::En,
            answers,
            defs: "",
            again_url: "/".to_string(),
        }
    }

    #[test]
    fn solutions_follow_every_puzzle_on_their_own_page() {
        let html = render(&sheet(6, Answers::Page));
        let key = html.find("class=\"page key\"").expect("answer page");
        let last_puzzle = html.find("detail 6").expect("sixth puzzle");
        assert!(last_puzzle < key);
        assert!(
            html.find("SOLUTION-1").expect("solution") > key,
            "no solution may appear before the answer page"
        );
    }

    #[test]
    fn footer_solutions_sit_upside_down_on_each_page() {
        let html = render(&sheet(12, Answers::Footer));
        assert_eq!(html.matches("class=\"upside-down\"").count(), 2);
        assert!(!html.contains("class=\"page key\""));
        assert!(
            html.contains("<strong>7</strong>"),
            "numbering continues on page two"
        );
    }

    #[test]
    fn a_sheet_without_answers_carries_none() {
        let html = render(&sheet(6, Answers::None));
        assert!(!html.contains("SOLUTION-"));
    }

    #[test]
    fn twelve_puzzles_make_two_pages() {
        let html = render(&sheet(12, Answers::None));
        assert_eq!(html.matches("<section class=\"page\">").count(), 2);
        assert!(html.contains("2/2"));
    }

    #[test]
    fn a_title_cannot_inject_markup() {
        let html = render(&sheet(1, Answers::None));
        assert!(html.contains("Forks &lt;b&gt;"));
        assert!(!html.contains("Forks <b>"));
    }
}
