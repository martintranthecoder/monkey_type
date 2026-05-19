# monkey_type

A terminal typing test written in Rust. Sit down, sip some coffee, watch the monkey, type fast.

```
                        .="=.
                      _/.-.-.\_     _
                     ( ( o o ) )    ))
                      |/  "  \|    //
      .-------.        \'---'/    //
     _|~~ ~~  |_       /`"""`\\  ((
   =(_|_______|_)=    / /_,_\ \\  \\
     |:::::::::|      \_\\_'__/ \  ))
     |:::::::[]|       /`  /`~\  |//
     |o=======.|      /   /    \  /
     `"""""""""`  ,--`,--'\/\    /
                   '-- "--'  '--'
```

## Features

- **Two test modes**: timed (run for N seconds) or word-count (type N words)
- **Two word lists**: `english` (top ~200) and `english_1k` (top 1000)
- **Live stats**: WPM and accuracy update as you type
- **Persistent history**: every completed run is appended to `~/Library/Application Support/monkey_type/history.json` (mac) or `$XDG_DATA_HOME/monkey_type/` (linux)
- **Tier feedback**: results screen rates your run from `more bananas needed` to `BANANAS!! top-tier ape`
- **Zero-setup**: word lists are embedded in the binary

## Install

From source (requires Rust 1.85+ for edition 2024):

```bash
git clone https://github.com/martintranthecoder/monkey_type
cd monkey_type
cargo install --path .
```

This drops a `monkey_type` binary in `~/.cargo/bin/`.

Or just run from the project directory:

```bash
cargo run --release
```

## Usage

```bash
monkey_type                            # 30-second timed test, english word list
monkey_type --time 60                  # 60-second timed test
monkey_type --words 25                 # type exactly 25 words
monkey_type --difficulty english_1k    # use the larger 1000-word list
monkey_type -t 15 -d english_1k        # short forms
monkey_type --help                     # see all flags
```

### Flags

| Flag                | Default     | What it does                              |
|---------------------|-------------|-------------------------------------------|
| `-t`, `--time`      | `30`        | Run a timed test for this many seconds    |
| `-w`, `--words`     | —           | Run a word-count test of this many words  |
| `-d`, `--difficulty`| `english`   | Word list: `english` or `english_1k`      |
| `-h`, `--help`      | —           | Print usage and exit                      |

`--time` and `--words` are mutually exclusive — the later flag wins.

## Keys

| Key            | Action                                          |
|----------------|-------------------------------------------------|
| any letter     | Type. The timer starts on your first keypress.  |
| `space`        | Finish the current word and advance             |
| `backspace`    | Erase the last character in the current word    |
| `tab`          | Restart the test with a fresh set of words      |
| `esc`          | Quit (no save)                                  |
| `ctrl+c`       | Quit                                            |
| `enter` / `tab`| (on results screen) Start a new test — same format, fresh words |

## How scoring works

- **WPM** is the standard `(correct_chars / 5) / minutes`. A "word" is 5 characters, so longer words count for more.
- **Accuracy** is `correct_chars / typed_chars * 100`. Backspacing reverts a character back to "untyped" — corrections are free.
- Only completed words contribute their trailing space to the correct-char count.
- The clock starts on your first keypress, not when the program launches. Take your time setting up.

## Tier ranks

The results screen prints one of these based on your WPM:

| WPM      | Rank                       |
|----------|----------------------------|
| 100+     | `BANANAS!! top-tier ape`   |
| 70–99    | `excellent -- silverback`  |
| 50–69    | `nice swing!`              |
| 30–49    | `keep climbing`            |
| < 30     | `more bananas needed`      |

## Project layout

```
src/
├── main.rs        # CLI args, terminal setup/teardown, panic hook
├── config/        # Config + TestMode, embedded word lists
├── game/          # GameState, key handling, WPM/accuracy math
├── stats/         # TestResult, History (persisted JSON)
└── ui/            # ratatui rendering and event loop
data/
├── english.json       # top ~200 English words
└── english_1k.json    # top 1000 English words
```

## Dependencies

- [`ratatui`](https://crates.io/crates/ratatui) — the TUI framework
- [`crossterm`](https://crates.io/crates/crossterm) — terminal input/output
- [`rand`](https://crates.io/crates/rand) — random word selection
- [`serde`](https://crates.io/crates/serde) + [`serde_json`](https://crates.io/crates/serde_json) — config/history persistence
- [`dirs`](https://crates.io/crates/dirs) — platform-correct config/data paths
- [`chrono`](https://crates.io/crates/chrono) — timestamps on saved runs

## Tips

- Run inside a terminal with at least 80 columns and 20 rows for the best layout. Smaller terminals work but the results layout gets cramped.
- The monkey ASCII art uses block/special characters. If you see weird boxes, switch your terminal to a font with good Unicode coverage (JetBrains Mono, Fira Code, Hack, Menlo, etc.).
- On macOS the history file lives at `~/Library/Application Support/monkey_type/history.json`. Delete it to reset your record.

## Acknowledgements

ASCII monkey-and-TV art by [jgs](http://www.ascii-art.de/).
