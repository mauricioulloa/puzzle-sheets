# Backlog

v0.1 is deliberately small: chess only, five levels, 25 themes, one layout.
What comes next should be decided by the teachers, parents and students who
use it, so most of this list is questions to answer with feedback rather than
features to build.

---

## Next up: the form

### 1. Chips instead of dropdowns

Level and theme as buttons that are always visible, so a teacher chooses in
one click each and sees every option without opening anything. Twenty-five
themes need to wrap gracefully on a phone.

### 2. A random button

After picking a level, one button that fills in the rest — a theme at random —
and makes the sheet. For someone who does not know what a skewer is yet, it
is the way in.

### 3. Old game or recent game

Say where each puzzle comes from in time. Every puzzle comes from a game
played on Lichess, so the oldest is from 2013; the classics of chess history
are not in the dataset. What can be shown is the game's date, fetched from
Lichess by its id, and whether titled players played it, which the `master`,
`masterVsMaster` and `superGM` themes already record.

---

## Learn from use

### 4. Put it in front of real classrooms

A handful of teachers and chess clubs, in Spanish and in English. What they
print, what they skip, and what they ask for is worth more than any item
below.

### 5. Know which sheets are made

Counters in the style of chess-puzzle-api's `/v1/usage`: levels, themes and
options chosen, per day, nothing about who. Enough to see which topics matter.

---

## The sheet

### 6. More per page, or fewer

ChessMint offers 1, 2, 4, 6 and 12 diagrams to a page. Six is a good default;
whether teachers want bigger boards for young children, or denser review
sheets, is a question for users.

### 7. Theme hints for the student

A small "Hint: fork" under a diagram, off by default, for a sheet on any
theme. The labels already exist in both languages.

---

## Beyond chess

### 8. A second puzzle type

The sheet already takes any diagram, prompt, detail and solution. Sudoku or
mazes are the natural next ones, generated here from a seed so a link stays
reproducible. Build it when someone asks for it, not before.

---

## Not doing, and why

- **Accounts, saved sheets, progress tracking.** A link is the saved sheet.
- **Criteria beyond rating and theme.** A cap on the pieces was tried and
  dropped: Lichess's rating and themes are the definitions, and a level is
  only a name for a band of rating.
- **Solutions at the foot of the page.** Two choices, a page of their own or
  none, are enough.
- **Server-side PDF.** The browser prints to PDF perfectly well.
- **Colour boards.** They waste ink and grey into mush on school printers.
