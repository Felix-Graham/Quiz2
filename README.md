# French Vocabulary Quiz

A minimalist, dark, keyboard-only terminal quiz for practicing French
vocabulary. Rewritten in Rust from the original Python prototype, packaged
to run on Windows, ChromeOS (Crostini), and NixOS.

No mouse, no buttons - every screen is driven by single keypresses or a
line of typed text.

## Install

### Linux / ChromeOS (Crostini)

```sh
./install.sh
```

This installs Rust for you if it's missing, builds a release binary, and
puts everything in `~/.local/share/frenchquiz` with a `frenchquiz` command
on your PATH.

### Windows

Double-click `install.bat`, or run it from a Command Prompt:

```bat
install.bat
```

It installs Rust if missing, builds the app, and copies it to
`%LOCALAPPDATA%\FrenchQuiz`, adding that folder to your user PATH.

### NixOS / Nix

```sh
nix run .            # try it without installing
nix profile install .    # install it for your user
```

or add it to your system/home-manager config by pointing at this flake's
`packages.default`.

**Caveat:** on Nix the binary lives in the read-only `/nix/store`, so it
can't keep its config or vocab files next to itself the way it does on
other platforms. The app detects this automatically and falls back to your
normal XDG locations instead (`~/.config/frenchquiz/config.toml` and
`~/.local/share/frenchquiz/vocab/`). Autoupdate via `git pull` also isn't
meaningful for a Nix store path - update with `nix flake update` in your
own checkout, or bump the flake input, instead.

### Building manually (any platform)

```sh
cargo build --release
# binary at target/release/frenchquiz(.exe)
```

## Usage

Run `frenchquiz` with no arguments for the menu, or jump straight in:

```
frenchquiz            # opens the home menu
frenchquiz all        # quiz on every vocab file, skip the picker
frenchquiz select     # choose which vocab file(s) to use
frenchquiz update     # git pull and exit
frenchquiz check      # print every vocab file's parsed pairs, for review
frenchquiz check <dir_or_file>   # check a specific folder instead
```

From the home menu:

| Key | Action |
|-----|--------|
| `s` | Start a quiz |
| `u` | Update (git pull) |
| `i` | Settings |
| `q` | Quit |

Quiz types: **typing** (you type the translation) or **multiple choice**.
Modes: continuous (until you stop), a set number of questions, maximum
(every loaded pair once), or timed.

## Vocab files

Vocab lives in the `vocab/` folder as `.txt` files. Two formats work:

**New format (recommended)** - one pair per line:

```
bonjour | hello
au revoir | goodbye
```

**Legacy format** - a 1-line title header, then alternating French/English
lines (compatible with the original Python quiz's vocab files):

```
<title line>

bonjour
hello
au revoir
goodbye
```

Files are grouped alphabetically by filename; a `01-`, `02-` prefix is a
handy way to order topics.

### Topics

Files named `vocab_<topic>-<part>.txt` - e.g. `vocab_1-1.txt`,
`vocab_1-2.txt`, `vocab_1-3.txt`, `vocab_2-1.txt` - are automatically
grouped by their topic number. When choosing vocab files, enter `t1` to
practice every file in topic 1 at once, `t2` for topic 2, and so on, or mix
topics and individual file numbers in one line (e.g. `t1 t2 5`).

Files that don't match that naming pattern - `essay_vocab.txt`,
`random_notes.txt`, anything without a `vocab_<N>-<M>` shape - are left out
of topic grouping and only selectable individually, exactly as before.

### If a quiz shows the wrong translation

Run `frenchquiz check` (or `frenchquiz check path/to/file.txt`) and read
through the output. For each file it shows:

- total lines in the file
- how many are blank
- how many are treated as a header (legacy format only)
- the remaining count, and whether that count is **even** - a necessary
  (but not sufficient) condition for a valid alternating french/english
  file, since each pair takes exactly two lines
- every parsed pair, numbered, so you can eyeball where things go wrong

An even count is a good sign but not a guarantee: if one translation gets
word-wrapped onto two lines (common when copying a vocab list out of Word,
Google Docs, or a PDF), the total still comes out even while every pair
after that point silently shifts. If `check` says "EVEN" but the numbered
list still looks wrong, look for a suspiciously short or truncated line
partway through - that's almost always the wrap point. The `french |
english` format doesn't have this failure mode at all, since each pair is
self-contained on one line; converting a problem file to it is the most
reliable fix.

## Settings

Open with `i` from the home menu:

- **Score display** - show a running score during quizzes.
- **Wide-range / fuzzy answers** - strips leading articles ("the", "a",
  "an"...) and accepts close typos (Levenshtein similarity above a
  threshold) instead of requiring an exact match.
- **Theme** - light or dark.
- **Direction** - ask French and expect English, or vice versa.
- **Streaks** - tracks consecutive days you've practiced; resets if you
  miss a day.
- **Update settings** - always update on launch, and/or ask before
  updating.

Settings and streak state are saved to `config.toml`.

## Project layout

```
src/
  main.rs     entry point, home menu, quiz-start flow
  config.rs   settings/streak persistence, path resolution
  vocab.rs    loading and parsing vocab files
  answer.rs   answer validation (exact / widened / fuzzy)
  streak.rs   daily streak bookkeeping
  quiz.rs     typing & multiple-choice quiz modes
  ui.rs       keyboard-only terminal UI helpers
  theme.rs    light/dark color palettes
  update.rs   git-pull autoupdate
install.sh    Linux / ChromeOS installer
install.bat   Windows installer
flake.nix     NixOS / Nix packaging
```
