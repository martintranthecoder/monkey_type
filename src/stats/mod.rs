use crate::config::{Config, TestMode};
use crate::game::{self, GameState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub wpm: f64,
    pub accuracy: f64,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub total_chars: usize,
    pub test_mode: TestMode,
    pub word_list: String,
    pub timestamp: DateTime<Utc>,
    pub duration_secs: f64,
}

impl TestResult {
    pub fn from_game(state: &GameState, config: &Config) -> Self {
        let correct_chars = state.correct_chars();
        let incorrect_chars = state.incorrect_chars();
        Self {
            wpm: game::calculate_wpm(state),
            accuracy: game::calculate_accuracy(state),
            correct_chars,
            incorrect_chars,
            total_chars: correct_chars + incorrect_chars,
            test_mode: config.test_mode,
            word_list: config.word_list.clone(),
            timestamp: Utc::now(),
            duration_secs: state.elapsed().as_secs_f64(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct History {
    pub results: Vec<TestResult>,
}

impl History {
    pub fn load() -> io::Result<Self> {
        let path = history_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path)?;
        serde_json::from_str(&text).map_err(io::Error::other)
    }

    pub fn save(&self) -> io::Result<()> {
        let path = history_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, text)
    }

    pub fn add(&mut self, result: TestResult) -> io::Result<()> {
        self.results.push(result);
        self.save()
    }

    #[allow(dead_code)]
    pub fn summary(&self) -> String {
        if self.results.is_empty() {
            return "No tests yet.".to_string();
        }
        let n = self.results.len();
        let avg_wpm: f64 = self.results.iter().map(|r| r.wpm).sum::<f64>() / n as f64;
        let best_wpm = self
            .results
            .iter()
            .map(|r| r.wpm)
            .fold(f64::NEG_INFINITY, f64::max);
        let best_acc = self
            .results
            .iter()
            .map(|r| r.accuracy)
            .fold(f64::NEG_INFINITY, f64::max);
        format!(
            "Tests: {n}  ·  Avg WPM: {avg_wpm:.1}  ·  Best WPM: {best_wpm:.1}  ·  Best Acc: {best_acc:.1}%"
        )
    }
}

fn history_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("monkey_type")
        .join("history.json")
}
