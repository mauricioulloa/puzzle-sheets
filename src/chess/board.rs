//! Chess diagrams as SVG, drawn the way printed puzzle books draw them: White
//! at the bottom, dark squares hatched rather than filled so they cost almost
//! no ink, and a small square beside the board saying who moves — filled at
//! the top when it is Black, empty at the bottom when it is White.

use crate::sheet::escape;

const SQUARE: usize = 45;
const BOARD: usize = 8 * SQUARE;
/// Room left of and below the board for the rank and file labels.
const MARGIN: usize = 16;
const MARKER: usize = 22;
const MARKER_GAP: usize = 10;
const WIDTH: usize = MARGIN + BOARD + MARKER_GAP + MARKER;
const HEIGHT: usize = BOARD + MARGIN;
/// Coordinates in viewBox units. At the 58 mm a sheet gives a board, 14 is
/// about 6 pt: readable by a child, and still inside the margin.
const LABEL_SIZE: usize = 14;

macro_rules! piece {
    ($code:literal) => {
        concat!(
            "<symbol id=\"p-",
            $code,
            "\" viewBox=\"0 0 45 45\">",
            include_str!(concat!("pieces/", $code, ".svg")),
            "</symbol>"
        )
    };
}

/// The piece set and the hatching, defined once per page and referenced by
/// every diagram.
pub const DEFS: &str = concat!(
    "<svg width=\"0\" height=\"0\" style=\"position:absolute\" aria-hidden=\"true\"><defs>",
    "<pattern id=\"hatch\" width=\"5\" height=\"5\" patternUnits=\"userSpaceOnUse\" patternTransform=\"rotate(45)\">",
    "<line x1=\"0\" y1=\"0\" x2=\"0\" y2=\"5\" stroke=\"#000\" stroke-width=\"0.9\"/></pattern>",
    piece!("wk"),
    piece!("wq"),
    piece!("wr"),
    piece!("wb"),
    piece!("wn"),
    piece!("wp"),
    piece!("bk"),
    piece!("bq"),
    piece!("br"),
    piece!("bb"),
    piece!("bn"),
    piece!("bp"),
    "</defs></svg>"
);

/// Draws the placement field of a FEN, with the side-to-move marker.
/// `label` is what a screen reader announces instead of the picture, and is
/// escaped here.
pub fn svg(fen: &str, white_to_move: bool, label: &str) -> String {
    let placement = fen.split_whitespace().next().unwrap_or_default();
    let mut out = format!(
        "<svg viewBox=\"0 0 {WIDTH} {HEIGHT}\" xmlns=\"http://www.w3.org/2000/svg\" role=\"img\"><title>{}</title>",
        escape(label)
    );

    for row in 0..8 {
        for col in 0..8 {
            if (row + col) % 2 == 1 {
                out.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{SQUARE}\" height=\"{SQUARE}\" fill=\"url(#hatch)\"/>",
                    MARGIN + col * SQUARE,
                    row * SQUARE
                ));
            }
        }
    }

    // FEN lists rank 8 first, files a to h.
    for (row, rank) in placement.split('/').take(8).enumerate() {
        let mut col = 0;
        for ch in rank.chars() {
            if let Some(empty) = ch.to_digit(10) {
                col += empty as usize;
                continue;
            }
            let colour = if ch.is_ascii_uppercase() { 'w' } else { 'b' };
            out.push_str(&format!(
                "<use href=\"#p-{colour}{}\" x=\"{}\" y=\"{}\" width=\"{SQUARE}\" height=\"{SQUARE}\"/>",
                ch.to_ascii_lowercase(),
                MARGIN + col * SQUARE,
                row * SQUARE
            ));
            col += 1;
        }
    }

    out.push_str(&format!(
        "<rect x=\"{MARGIN}\" y=\"0\" width=\"{BOARD}\" height=\"{BOARD}\" fill=\"none\" stroke=\"#000\" stroke-width=\"1.5\"/>"
    ));

    let (marker_y, marker_fill) = if white_to_move {
        (BOARD - MARKER, "#fff")
    } else {
        (0, "#000")
    };
    out.push_str(&format!(
        "<rect x=\"{}\" y=\"{marker_y}\" width=\"{MARKER}\" height=\"{MARKER}\" fill=\"{marker_fill}\" stroke=\"#000\" stroke-width=\"1.2\"/>",
        MARGIN + BOARD + MARKER_GAP
    ));

    for i in 0..8 {
        out.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" font-size=\"{LABEL_SIZE}\" text-anchor=\"middle\" font-family=\"sans-serif\">{}</text>",
            MARGIN / 2,
            i * SQUARE + SQUARE / 2 + LABEL_SIZE / 3,
            8 - i
        ));
        out.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" font-size=\"{LABEL_SIZE}\" text-anchor=\"middle\" font-family=\"sans-serif\">{}</text>",
            MARGIN + i * SQUARE + SQUARE / 2,
            BOARD + 13,
            char::from(b'a' + i as u8)
        ));
    }

    out.push_str("</svg>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    #[test]
    fn draws_every_piece_once() {
        assert_eq!(svg(START, true, "").matches("<use ").count(), 32);
        assert_eq!(
            svg("8/8/8/8/8/8/8/K6k w - - 0 1", true, "")
                .matches("<use ")
                .count(),
            2
        );
    }

    #[test]
    fn white_always_sits_at_the_bottom() {
        // The white king on e1: bottom row, fifth file, whoever moves.
        for white_to_move in [true, false] {
            assert!(svg(START, white_to_move, "").contains("href=\"#p-wk\" x=\"196\" y=\"315\""));
        }
    }

    #[test]
    fn the_marker_says_who_moves() {
        let marker = |board: &str| {
            let at = board
                .find(&format!("x=\"{}\"", MARGIN + BOARD + MARKER_GAP))
                .unwrap();
            board[at..].split("/>").next().unwrap().to_string()
        };
        let white = marker(&svg(START, true, ""));
        assert!(white.contains("y=\"338\"") && white.contains("fill=\"#fff\""));
        let black = marker(&svg(START, false, ""));
        assert!(black.contains("y=\"0\"") && black.contains("fill=\"#000\""));
    }

    #[test]
    fn a_screen_reader_hears_the_label_not_the_picture() {
        let board = svg(START, true, "White to move. <Mate> in 1");
        assert!(board.contains("role=\"img\"><title>White to move. &lt;Mate&gt; in 1</title>"));
    }

    #[test]
    fn dark_squares_are_hatched_not_filled() {
        let board = svg(START, true, "");
        assert_eq!(board.matches("url(#hatch)").count(), 32);
        assert!(DEFS.contains("<pattern id=\"hatch\""));
        assert_eq!(DEFS.matches("<symbol ").count(), 12);
    }
}
