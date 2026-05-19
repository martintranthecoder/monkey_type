use crossterm::event::{KeyCode, KeyEvent};
use rand::rng;
use rand::seq::IndexedRandom;
use serde::Deserialize;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharResult {
    Correct,
    Incorrect,
    Untyped,
}

pub struct GameState {
    pub words: Vec<String>,
    pub input: String,
    pub current_word: usize,
    pub char_results: Vec<Vec<CharResult>>,
    pub started_at: Option<Instant>,
    pub finished_at: Option<Instant>,
    pub test_duration: Option<Duration>,
}

impl GameState {
    pub fn new(words: Vec<String>, duration: Option<Duration>) -> Self {
        let char_results = words
            .iter()
            .map(|w| vec![CharResult::Untyped; w.chars().count()])
            .collect();
        Self {
            words,
            input: String::new(),
            current_word: 0,
            char_results,
            started_at: None,
            finished_at: None,
            test_duration: duration,
        }
    }

    pub fn elapsed(&self) -> Duration {
        match (self.started_at, self.finished_at) {
            (Some(start), Some(end)) => end.duration_since(start),
            (Some(start), None) => {
                let raw = start.elapsed();
                match self.test_duration {
                    Some(d) if raw > d => d,
                    _ => raw,
                }
            }
            _ => Duration::ZERO,
        }
    }

    pub fn correct_chars(&self) -> usize {
        let mut count = 0;
        for (i, results) in self.char_results.iter().enumerate() {
            let typed = if i < self.current_word {
                results.len()
            } else if i == self.current_word {
                self.input.chars().count().min(results.len())
            } else {
                0
            };
            for r in results.iter().take(typed) {
                if *r == CharResult::Correct {
                    count += 1;
                }
            }
            // Count the space after each fully-completed word as a correct char
            if i < self.current_word {
                count += 1;
            }
        }
        count
    }

    pub fn incorrect_chars(&self) -> usize {
        let mut count = 0;
        for (i, results) in self.char_results.iter().enumerate() {
            let typed = if i < self.current_word {
                results.len()
            } else if i == self.current_word {
                self.input.chars().count().min(results.len())
            } else {
                0
            };
            for r in results.iter().take(typed) {
                if *r == CharResult::Incorrect {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn total_typed(&self) -> usize {
        self.correct_chars() + self.incorrect_chars()
    }
}

#[derive(Deserialize)]
struct WordListFile {
    words: Vec<String>,
}

pub fn load_words_from_str(json: &str, count: usize) -> Vec<String> {
    let parsed: WordListFile = serde_json::from_str(json).expect("invalid word list JSON");
    let mut rng = rng();
    if parsed.words.is_empty() {
        return Vec::new();
    }
    if count <= parsed.words.len() {
        parsed
            .words
            .choose_multiple(&mut rng, count)
            .cloned()
            .collect()
    } else {
        (0..count)
            .map(|_| parsed.words.choose(&mut rng).unwrap().clone())
            .collect()
    }
}

pub fn handle_key(state: &mut GameState, key: KeyEvent) {
    if state.finished_at.is_some() || state.current_word >= state.words.len() {
        return;
    }
    match key.code {
        KeyCode::Char(' ') => {
            if state.started_at.is_none() {
                return;
            }
            state.current_word += 1;
            state.input.clear();
        }
        KeyCode::Char(c) => {
            if state.started_at.is_none() {
                state.started_at = Some(Instant::now());
            }
            let word_chars: Vec<char> = state.words[state.current_word].chars().collect();
            let idx = state.input.chars().count();
            state.input.push(c);
            if idx < word_chars.len() {
                state.char_results[state.current_word][idx] = if word_chars[idx] == c {
                    CharResult::Correct
                } else {
                    CharResult::Incorrect
                };
            }
        }
        KeyCode::Backspace => {
            if state.input.is_empty() {
                return;
            }
            let removed_idx = state.input.chars().count() - 1;
            state.input.pop();
            let word_len = state.words[state.current_word].chars().count();
            if removed_idx < word_len {
                state.char_results[state.current_word][removed_idx] = CharResult::Untyped;
            }
        }
        _ => {}
    }
}

pub fn is_finished(state: &GameState) -> bool {
    if let Some(d) = state.test_duration {
        match state.started_at {
            Some(start) => start.elapsed() >= d,
            None => false,
        }
    } else {
        state.current_word >= state.words.len()
    }
}

pub fn calculate_wpm(state: &GameState) -> f64 {
    let elapsed = state.elapsed().as_secs_f64();
    if elapsed <= 0.0 {
        return 0.0;
    }
    let correct = state.correct_chars() as f64;
    (correct / 5.0) / (elapsed / 60.0)
}

pub fn calculate_accuracy(state: &GameState) -> f64 {
    let total = state.total_typed();
    if total == 0 {
        return 100.0;
    }
    (state.correct_chars() as f64 / total as f64) * 100.0
}
