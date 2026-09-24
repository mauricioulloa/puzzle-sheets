# Backlog

Ordered by priority, not by effort. v0.1 is deliberately small: chess only,
five levels, 25 themes, one layout. What comes next should be decided by the
teachers, parents and students who use it, so much of this list is questions
to answer with feedback rather than features to build.

Items needing Mauri's own accounts or contacts cannot be done unattended; they
are marked **[needs you]**.

---

## 1 · Next up: the form

### 1. Chips instead of dropdowns

Level and theme as buttons that are always visible, so a teacher chooses in
one click each and sees every option without opening anything. Twenty-five
themes need to wrap gracefully on a phone.

### 2. A random button

After picking a level, one button that fills in the rest — a theme at random —
and makes the sheet. For someone who does not know what a skewer is yet, it
is the way in.

### 3. Choose how much the sheet says

From user feedback, September 2026. Two requests pull in opposite directions:
make the instructions more descriptive for children, and hide the kind of
exercise, because the theme in the subtitle and the goal under each board
give the idea away. Both are right for different readers, so the teacher
chooses on the form, before generating, how descriptive the sheet is:

- **Hidden:** number, board and side to move only; no theme and no goal.
- **Standard:** today's sheet, with the goal ("Mate in 2") and the theme.
- **For children:** full sentences — "White to move. Find the checkmate in
  two moves." — and a short line on what the theme means.

---

## 2 · Learn from use

### 4. Put it in front of real classrooms — **[needs you]**

A handful of teachers and chess clubs, in Spanish and in English. What they
print, what they skip, and what they ask for is worth more than any item
below.

### 5. Know which sheets are made

Counters in the style of chess-puzzle-api's `/v1/usage`: levels, themes and
options chosen, per day, nothing about who. Enough to see which topics matter.

---

## 3 · The sheet

### 6. More per page, or fewer

ChessMint offers 1, 2, 4, 6 and 12 diagrams to a page. Six is a good default;
whether teachers want bigger boards for young children, or denser review
sheets, is a question for users.

---

## 4 · Discoverability

The same diagnosis as chess-puzzle-api's: one page, no `robots.txt`, no
sitemap, no favicon, no Open Graph. A link shared in a teachers' group chat
previews as nothing.

### 7. The basics

`robots.txt`, `sitemap.xml`, a favicon, `<link rel="canonical">`, Open Graph
tags in both languages, and a fixed preview image — a board and the site
name. Sheets themselves should be `noindex`: each is a list of ids, not
content.

### 8. Submit to the MCP registries — **[needs you]**

Together with chess-puzzle-api's own submission ("Submit to the MCP
registries" in its backlog): the official registry, Smithery, Glama, mcp.so
and PulseMCP.

---

## 5 · Later

### 9. Fewer requests per sheet

Once chess-puzzle-api can look puzzles up in batches ("Look puzzles up in
batches" in its backlog), a twelve-puzzle sheet drops from 25 requests to
three.

### 10. A verification script for production

Like chess-puzzle-api's `scripts/verify_production.py`: make a sheet on the
live service, open its link, and check that the puzzles and solutions it
prints match the API's.

### 11. A second puzzle type

The sheet already takes any diagram, prompt, detail and solution. Sudoku or
mazes are the natural next ones, generated here from a seed so a link stays
reproducible. Build it when someone asks for it, not before.

---

## Not doing, and why

- **Accounts, saved sheets, progress tracking.** A link is the saved sheet.
- **Criteria beyond rating and theme.** A cap on the pieces was tried and
  dropped: Lichess's rating and themes are the definitions, and a level is
  only a name for a band of rating.
- **Rejecting unknown parameters.** chess-puzzle-api turns a misspelt
  parameter into a `400`, which suits a program. A sheet's link is shared by
  people, through apps that append `fbclid` or `utm_*` to it, and it has to
  keep printing.
- **Solutions at the foot of the page.** Two choices, a page of their own or
  none, are enough.
- **Server-side PDF.** The browser prints to PDF perfectly well.
- **Colour boards.** They waste ink and grey into mush on school printers.
