use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    FrenchToEnglish,
    EnglishToFrench,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub score_enabled: bool,
    pub wide_range_enabled: bool,
    pub fuzzy_threshold: u8,
    pub theme: Theme,
    pub direction: Direction,
    pub always_update: bool,
    pub ask_before_update: bool,
    pub streaks_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            score_enabled: true,
            wide_range_enabled: true,
            fuzzy_threshold: 80,
            theme: Theme::Dark,
            direction: Direction::FrenchToEnglish,
            always_update: false,
            ask_before_update: true,
            streaks_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StreakState {
    pub current: u32,
    pub best: u32,
    pub last_day: i64,
}

impl Default for StreakState {
    fn default() -> Self {
        StreakState {
            current: 0,
            best: 0,
            last_day: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub alias: Option<String>,
    pub settings: Settings,
    pub streak: StreakState,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            alias: None,
            settings: Settings::default(),
            streak: StreakState::default(),
        }
    }
}

pub fn install_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn is_writable_dir(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    let probe = dir.join(".frenchquiz-write-test");
    match fs::File::create(&probe) {
        Ok(_) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

pub fn config_path() -> PathBuf {
    let beside_exe = install_dir().join("config.toml");
    if beside_exe.exists() || is_writable_dir(&install_dir()) {
        return beside_exe;
    }
    // Nix store is read-only, use XDG config instead
    let fallback_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("frenchquiz");
    let _ = fs::create_dir_all(&fallback_dir);
    fallback_dir.join("config.toml")
}

pub fn vocab_dir() -> PathBuf {
    let dir = install_dir();
    let beside_exe = dir.join("vocab");
    if beside_exe.is_dir() {
        return beside_exe;
    }
    // Nix layout: bin and share are siblings
    let nix_share = dir
        .parent()
        .map(|p| p.join("share").join("frenchquiz").join("vocab"));
    if let Some(p) = &nix_share {
        if p.is_dir() {
            return p.clone();
        }
    }
    let data_fallback = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("frenchquiz")
        .join("vocab");
    if data_fallback.is_dir() {
        return data_fallback;
    }
    if is_writable_dir(&dir) {
        beside_exe
    } else {
        data_fallback
    }
}

impl Config {
    pub fn load() -> Config {
        let path = config_path();
        match fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_default(),
            Err(_) => {
                let cfg = Config::default();
                let _ = cfg.save();
                cfg
            }
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let text = toml::to_string_pretty(self).unwrap_or_default();
        fs::write(config_path(), text)
    }
}

pub fn ensure_vocab_dir_exists(dir: &Path) -> std::io::Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}
