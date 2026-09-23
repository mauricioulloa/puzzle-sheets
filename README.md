# puzzle-sheets

Printable chess worksheets for students and educators, in Spanish and
English. Free, black and white, and in the public domain.

**Live at [puzzles.mauriulloa.com](https://puzzles.mauriulloa.com)**

By [Mauri Ulloa](https://mauriulloa.com)

## What a sheet looks like

Six puzzles to a page. Under each diagram:

- who moves and what to find, the way chess sites put it: *White to move ·
  Mate in 2*;
- the position's FEN, to set it up in any chess program;
- a line for the answer.

Solutions go on their own page so a teacher can keep them, upside down at the
foot of each page as puzzle books do, or nowhere at all for a student working
alone. Boards are drawn from the solver's side; Spanish sheets use Spanish
piece letters (R, D, T, A, C).

## Presets

A teacher picks "Mate in 1", not an Elo band and a theme id. Each preset
caps the number of pieces on the board, because a low rating does not mean a
simple position: mate-in-ones under 1000 have a median of nineteen pieces,
and a beginner has to scan every one.

| | |
| --- | --- |
| `mate-in-1` | first checkmates, at most 12 pieces |
| `hanging-pieces` | take the undefended piece |
| `forks` | the best first tactic to learn |
| `back-rank` | the king trapped behind its pawns |
| `mate-in-2` | the opponent's reply has to be seen |
| `pins-and-skewers` | attacks along a line |
| `pawn-endgames` | kings and pawns, at most 8 pieces |

## Links, not files

A sheet is its URL. `/sheet/new?preset=forks` picks puzzles and redirects to
`/sheet?ids=...`, and that link always prints the same puzzles with the same
answer key. Nothing is stored.

```
/sheet/new?preset=forks&count=12&lang=es&answers=footer&title=3º básico
/sheet/new?themes=pin&rating=1100&maxPieces=12
/sheet?ids=00008,00014
```

## For language models

`POST /mcp` is a Model Context Protocol server with `list_presets` and
`create_worksheet`, which returns a printable link. An assistant helping a
teacher, a parent or a student can hand them a sheet instead of describing
positions in chat. [`/llms.txt`](https://puzzles.mauriulloa.com/llms.txt)
describes the rest.

## How it works

Puzzles come from [chess-puzzle-api](https://github.com/mauricioulloa/chess-puzzle-api),
which serves the Lichess puzzle database. This service owns the page: it
draws the boards as SVG, lays out the sheet for A4 and Letter alike, and
leaves PDF to the browser's own print dialog.

The sheet itself does not know it is chess. A puzzle type supplies a diagram,
a prompt, a detail line and a solution; the layout, answer key and presets
work the same for whatever comes next.

## Self-hosting

```bash
cargo run -- serve
```

It reads `CHESS_API_URL` (defaults to the public API), `CHESS_API_KEY`,
`PUBLIC_URL`, `BIND_ADDR` and `MCP_ALLOWED_HOSTS`. Without a key every sheet
shares the API's anonymous rate limit, and a sheet costs two requests per
puzzle.

```bash
cargo test && cargo clippy --all-targets
```

Tests run against a stand-in for the API: no network needed.

## Feedback

This is a first version, and what comes next depends on the people using it.
A topic that is missing, a level that feels wrong, a layout that does not
print well in your school — please
[open an issue](https://github.com/mauricioulloa/puzzle-sheets/issues).
What is already planned is in [BACKLOG.md](BACKLOG.md).

## Attribution

Puzzle data from the [Lichess open database](https://database.lichess.org/),
public domain under CC0. Lichess is free and ad-free —
[consider supporting them](https://lichess.org/patron). This project is not
affiliated with or endorsed by Lichess.

Chess pieces by
[Colin M.L. Burnett](https://en.wikipedia.org/wiki/User:Cburnett),
[CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/).

## License

[MIT](LICENSE) © [Mauri Ulloa](https://mauriulloa.com)
