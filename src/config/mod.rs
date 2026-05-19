use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestMode {
    Timed(u64),
    WordCount(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub test_mode: TestMode,
    pub word_list: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            test_mode: TestMode::Timed(30),
            word_list: "english".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> io::Result<Self> {
        let path = config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path)?;
        let cfg = serde_json::from_str(&text).map_err(io::Error::other)?;
        Ok(cfg)
    }

    #[allow(dead_code)]
    pub fn save(&self) -> io::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, text)
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("monkey_type")
        .join("config.json")
}

const ENGLISH_JSON: &str = include_str!("../../data/english.json");
const ENGLISH_1K_JSON: &str = include_str!("../../data/english_1k.json");

pub fn word_list_json(name: &str) -> &'static str {
    match name {
        "english_1k" => ENGLISH_1K_JSON,
        _ => ENGLISH_JSON,
    }
}
