mod config;
mod game;
mod stats;
mod ui;

use crate::config::{Config, TestMode};
use crate::stats::History;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::Duration;

fn parse_args(config: &mut Config) {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--time" | "-t" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse::<u64>().ok()) {
                    config.test_mode = TestMode::Timed(v);
                    i += 2;
                    continue;
                }
            }
            "--words" | "-w" => {
                if let Some(v) = args.get(i + 1).and_then(|s| s.parse::<usize>().ok()) {
                    config.test_mode = TestMode::WordCount(v);
                    i += 2;
                    continue;
                }
            }
            "--difficulty" | "-d" => {
                if let Some(v) = args.get(i + 1) {
                    config.word_list = v.clone();
                    i += 2;
                    continue;
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }
}

fn print_help() {
    println!("monkey_type — a terminal typing test\n");
    println!("USAGE:");
    println!("    monkey_type [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    -t, --time <SECONDS>        Run a timed test (default: 30)");
    println!("    -w, --words <COUNT>         Run a word-count test");
    println!("    -d, --difficulty <NAME>     Word list: english | english_1k");
    println!("    -h, --help                  Show this help");
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut config = Config::load().unwrap_or_default();
    parse_args(&mut config);

    // Make sure the terminal is restored even on panic.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        default_hook(info);
    }));

    let json = config::word_list_json(&config.word_list);
    let (count, duration) = match config.test_mode {
        TestMode::Timed(secs) => {
            let count = ((secs as usize) * 6).clamp(50, 2000);
            (count, Some(Duration::from_secs(secs)))
        }
        TestMode::WordCount(n) => (n.max(1), None),
    };
    let words = game::load_words_from_str(json, count);
    let mut state = game::GameState::new(words, duration);

    let mut terminal = setup_terminal()?;
    let outcome = ui::run(&mut terminal, &mut state, &config);
    restore_terminal()?;

    match outcome {
        Ok(results) => {
            if !results.is_empty() {
                if let Ok(mut history) = History::load() {
                    for r in &results {
                        history.results.push(r.clone());
                    }
                    let _ = history.save();
                }
            }
            if let Some(last) = results.last() {
                let session = if results.len() > 1 {
                    format!("  [{} runs this session]", results.len())
                } else {
                    String::new()
                };
                println!(
                    "WPM: {:.1}  Accuracy: {:.1}%  Time: {:.1}s  ({}/{} chars){session}",
                    last.wpm,
                    last.accuracy,
                    last.duration_secs,
                    last.correct_chars,
                    last.total_chars,
                );
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {e}");
            Err(e)
        }
    }
}
