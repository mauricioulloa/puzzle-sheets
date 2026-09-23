# puzzle-sheets

Printable chess worksheets for students and educators, in Spanish and
English. Free, light on ink, and in the public domain.

**Live at [puzzles.mauriulloa.com](https://puzzles.mauriulloa.com)**

By [Mauri Ulloa](https://mauriulloa.com)

## What a sheet looks like

Bare, the way a printed puzzle book looks, and light on ink. Six puzzles to a
page, each with:

- its number above the diagram;
- a board with White at the bottom and hatched dark squares, and a small
  square beside it saying who moves: filled at the top for Black, empty at
  the bottom for White;
- what to find, the way chess sites put it: *Mate in 2*;
- the position's FEN, to set it up in any chess program.

Solutions sit upside down at the foot of each page, go on a page of their own
after each page of puzzles (print double-sided and they land on the back), or
are left out for a student working alone. Spanish sheets use Spanish piece letters (R, D, T, A,
C).

## Difficulty and theme

A sheet takes one of five levels and, optionally, one of 25 themes: mates in
one to five, sacrifices, the four endgame families, forks, pins, skewers and
the rest of the tactical vocabulary.

| Level | Rating | Pieces |
| --- | --- | --- |
| `beginner` | 400–1000 | at most 12 |
| `novice` | 900–1300 | at most 16 |
| `intermediate` | 1300–1700 | any |
| `advanced` | 1700–2100 | any |
| `expert` | 2100–2800 | any |

The two easiest levels cap the pieces because a low rating does not mean a
simple position: mate-in-ones under 1000 have a median of nineteen pieces, and
a beginner has to scan every one. When a theme barely exists on a sparse board
(a kingside attack needs pieces to attack with), the sheet is filled without
the cap rather than left half empty.

## Links, not files

A sheet is its URL. `/sheet/new` picks puzzles and redirects to
`/sheet?ids=...`, and that link always prints the same puzzles with the same
solutions. Nothing is stored.

```
/sheet/new?level=beginner&theme=fork&count=12&lang=es&title=3º básico
/sheet/new?level=advanced&answers=page
/sheet?ids=00008,00014
```

## For language models

`POST /mcp` is a Model Context Protocol server with `list_options` and
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
a prompt, a detail line and a solution; the layout and the solutions work the
same for whatever comes next.

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
