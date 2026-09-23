//! Chess diagrams as SVG, drawn for a black-and-white printer: white and
//! light-grey squares, a hard border, and the Cburnett pieces as vectors so
//! they stay sharp at any size.

const SQUARE: usize = 45;
/// Room left of and below the board for the rank and file labels.
const MARGIN: usize = 16;
const SIZE: usize = MARGIN + 8 * SQUARE;

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

/// The piece set, defined once per page and referenced by every diagram.
pub const DEFS: &str = concat!(
    "<svg width=\"0\" height=\"0\" style=\"position:absolute\" aria-hidden=\"true\"><defs>",
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

/// Draws the placement field of a FEN. `flipped` puts Black at the bottom,
/// which is how a puzzle is shown when Black is the side to solve it.
pub fn svg(fen: &str, flipped: bool) -> String {
    let placement = fen.split_whitespace().next().unwrap_or_default();
    let mut out = format!(
        "<svg viewBox=\"0 0 {SIZE} {SIZE}\" xmlns=\"http://www.w3.org/2000/svg\" role=\"img\">"
    );

    for row in 0..8 {
        for col in 0..8 {
            if (row + col) % 2 == 1 {
                out.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{SQUARE}\" height=\"{SQUARE}\" fill=\"#c8c8c8\"/>",
                    MARGIN + col * SQUARE,
                    row * SQUARE
                ));
            }
        }
    }

    // FEN lists rank 8 first, files a to h.
    for (rank_index, rank) in placement.split('/').take(8).enumerate() {
        let mut file_index = 0;
        for ch in rank.chars() {
            if let Some(empty) = ch.to_digit(10) {
                file_index += empty as usize;
                continue;
            }
            let colour = if ch.is_ascii_uppercase() { 'w' } else { 'b' };
            let (row, col) = if flipped {
                (7 - rank_index, 7 - file_index)
            } else {
                (rank_index, file_index)
            };
            out.push_str(&format!(
                "<use href=\"#p-{colour}{}\" x=\"{}\" y=\"{}\" width=\"{SQUARE}\" height=\"{SQUARE}\"/>",
                ch.to_ascii_lowercase(),
                MARGIN + col * SQUARE,
                row * SQUARE
            ));
            file_index += 1;
        }
    }

    out.push_str(&format!(
        "<rect x=\"{MARGIN}\" y=\"0\" width=\"{w}\" height=\"{w}\" fill=\"none\" stroke=\"#000\" stroke-width=\"2\"/>",
        w = 8 * SQUARE
    ));

    for i in 0..8 {
        let rank = if flipped { i + 1 } else { 8 - i };
        let file = char::from(if flipped {
            b'h' - i as u8
        } else {
            b'a' + i as u8
        });
        out.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" font-size=\"13\" text-anchor=\"middle\" font-family=\"sans-serif\">{rank}</text>",
            MARGIN / 2,
            i * SQUARE + SQUARE / 2 + 5
        ));
        out.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" font-size=\"13\" text-anchor=\"middle\" font-family=\"sans-serif\">{file}</text>",
            MARGIN + i * SQUARE + SQUARE / 2,
            8 * SQUARE + 13
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
        assert_eq!(svg(START, false).matches("<use ").count(), 32);
        assert_eq!(
            svg("8/8/8/8/8/8/8/K6k w - - 0 1", false)
                .matches("<use ")
                .count(),
            2
        );
    }

    #[test]
    fn white_sits_at_the_bottom_unless_flipped() {
        // The white king on e1: bottom row, fifth file.
        let upright = svg(START, false);
        assert!(upright.contains("href=\"#p-wk\" x=\"196\" y=\"315\""));
        // Flipped, e1 moves to the top row and the fourth column from the left.
        let flipped = svg(START, true);
        assert!(flipped.contains("href=\"#p-wk\" x=\"151\" y=\"0\""));
    }

    #[test]
    fn labels_follow_the_orientation() {
        let upright = svg(START, false);
        let flipped = svg(START, true);
        let first_file = |board: &str| {
            board
                .split("</text>")
                .nth(1)
                .unwrap()
                .chars()
                .last()
                .unwrap()
        };
        assert_eq!(first_file(&upright), 'a');
        assert_eq!(first_file(&flipped), 'h');
    }

    #[test]
    fn defines_all_twelve_pieces() {
        assert_eq!(DEFS.matches("<symbol ").count(), 12);
    }
}
