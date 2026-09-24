//! Chess puzzles as worksheet items.

pub mod api;
pub mod board;
pub mod options;

use crate::i18n::{Lang, Text};
use crate::sheet::Item;
use crate::worksheet::{Invalid, WorksheetError};
use api::{ChessApi, ChessApiError, Puzzle, Solution};
use futures_util::future::try_join_all;

/// Fetches the puzzles in the order given, each with its solution.
pub async fn items(
    api: &ChessApi,
    ids: &[String],
    lang: Lang,
) -> Result<Vec<Item>, WorksheetError> {
    let fetches = ids.iter().map(|id| async move {
        let (puzzle, solution) =
            tokio::try_join!(api.puzzle(id), api.solution(id)).map_err(|err| match err {
                ChessApiError::NotFound => Invalid::UnknownPuzzle(id.clone()).into(),
                other => WorksheetError::from(other),
            })?;
        Ok::<_, WorksheetError>(item(&puzzle, &solution, lang))
    });
    try_join_all(fetches).await
}

pub fn item(puzzle: &Puzzle, solution: &Solution, lang: Lang) -> Item {
    let white_to_move = puzzle.solver_color == "white";
    Item {
        diagram: board::svg(&puzzle.position_fen, white_to_move),
        prompt: goal(&puzzle.themes, lang.text()),
        detail: Some(puzzle.position_fen.clone()),
        solution: numbered(&solution.solution_san, white_to_move, lang),
    }
}

/// What the student is asked to achieve, in the words chess sites use.
fn goal(themes: &[String], text: &Text) -> String {
    let has = |name: &str| themes.iter().any(|theme| theme == name);
    for moves in 1..=5 {
        if has(&format!("mateIn{moves}")) {
            return format!("{} {moves}", text.goal_mate_in);
        }
    }
    let goal = if has("mate") {
        text.goal_mate
    } else if has("crushing") {
        text.goal_crushing
    } else if has("advantage") {
        text.goal_advantage
    } else if has("equality") {
        text.goal_equality
    } else {
        text.goal_best_move
    };
    goal.to_string()
}

/// Numbers the line from 1, the way puzzle books print it: `1. Rxe7 Qb1+
/// 2. Nc1`, or `1... Qb1+ 2. Nc1` when Black moves first.
fn numbered(moves: &[String], white_first: bool, lang: Lang) -> String {
    let mut parts = Vec::with_capacity(moves.len());
    let mut number = 1;
    let mut white_to_move = white_first;
    for (index, san) in moves.iter().enumerate() {
        let san = localise(san, lang);
        if white_to_move {
            parts.push(format!("{number}. {san}"));
        } else {
            parts.push(if index == 0 {
                format!("{number}... {san}")
            } else {
                san
            });
            number += 1;
        }
        white_to_move = !white_to_move;
    }
    parts.join(" ")
}

/// Spanish schools teach Spanish piece letters: Rey, Dama, Torre, Alfil,
/// Caballo. Squares and castling are the same in both.
fn localise(san: &str, lang: Lang) -> String {
    match lang {
        Lang::En => san.to_string(),
        Lang::Es => san
            .chars()
            .map(|ch| match ch {
                'K' => 'R',
                'Q' => 'D',
                'R' => 'T',
                'B' => 'A',
                'N' => 'C',
                other => other,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn states_the_goal_like_a_chess_site() {
        let en = Lang::En.text();
        assert_eq!(
            goal(&strings(&["mate", "mateIn2", "short"]), en),
            "Mate in 2"
        );
        assert_eq!(goal(&strings(&["mate", "veryLong"]), en), "Checkmate");
        assert_eq!(goal(&strings(&["fork", "crushing"]), en), "Win decisively");
        assert_eq!(goal(&strings(&["fork"]), en), "Find the best move");
        assert_eq!(goal(&strings(&["mateIn1"]), Lang::Es.text()), "Mate en 1");
    }

    #[test]
    fn numbers_the_line_from_the_solver() {
        let line = strings(&["Rxe7", "Qb1+", "Nc1", "Qxc1+", "Qxc1"]);
        assert_eq!(
            numbered(&line, true, Lang::En),
            "1. Rxe7 Qb1+ 2. Nc1 Qxc1+ 3. Qxc1"
        );
        assert_eq!(
            numbered(&line, false, Lang::En),
            "1... Rxe7 2. Qb1+ Nc1 3. Qxc1+ Qxc1"
        );
    }

    #[test]
    fn spanish_sheets_use_spanish_piece_letters() {
        assert_eq!(localise("Qxc1+", Lang::Es), "Dxc1+");
        assert_eq!(localise("Nf3", Lang::Es), "Cf3");
        assert_eq!(localise("exd8=Q#", Lang::Es), "exd8=D#");
        assert_eq!(localise("O-O-O", Lang::Es), "O-O-O");
        assert_eq!(localise("Bb5", Lang::Es), "Ab5", "the b file must survive");
    }

    #[test]
    fn a_black_puzzle_says_so_with_the_marker() {
        let puzzle = Puzzle {
            id: "x".into(),
            position_fen: "8/8/8/8/8/8/8/K6k b - - 0 1".into(),
            solver_color: "black".into(),
            themes: strings(&["mateIn1"]),
        };
        let solution = Solution {
            solution_san: strings(&["Kg2"]),
        };
        let item = item(&puzzle, &solution, Lang::Es);
        assert_eq!(item.prompt, "Mate en 1");
        assert_eq!(item.detail.as_deref(), Some(puzzle.position_fen.as_str()));
        assert_eq!(item.solution, "1... Rg2");
        assert_eq!(item.diagram, board::svg(&puzzle.position_fen, false));
    }
}
