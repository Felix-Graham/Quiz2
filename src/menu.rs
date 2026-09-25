use crate::config::{Config, Direction, Theme};
use crate::ui::Ui;
use crate::update;
use crate::vocab;
use std::path::PathBuf;

pub fn settings_menu(cfg: &mut Config, ui: &mut Ui) {
    loop {
        ui.header("Settings");
        ui.dim(&format!(
            "  score: {} | wide-range: {} | theme: {:?} | direction: {} | streaks: {}",
            on_off(cfg.settings.score_enabled),
            on_off(cfg.settings.wide_range_enabled),
            cfg.settings.theme,
            direction_label(cfg.settings.direction),
            on_off(cfg.settings.streaks_enabled),
        ));
        println!();
        match ui.menu(&[
            ('s', "Toggle score display"),
            ('w', "Toggle wide-range / fuzzy answers"),
            ('t', "Toggle light / dark theme"),
            ('d', "Toggle question direction (FR<->EN)"),
            ('k', "Toggle streak tracking"),
            ('u', "Update settings"),
            ('q', "Back"),
        ]) {
            's' => cfg.settings.score_enabled = !cfg.settings.score_enabled,
            'w' => cfg.settings.wide_range_enabled = !cfg.settings.wide_range_enabled,
            't' => {
                cfg.settings.theme = match cfg.settings.theme {
                    Theme::Dark => Theme::Light,
                    Theme::Light => Theme::Dark,
                };
                ui.set_theme(cfg.settings.theme);
            }
            'd' => {
                cfg.settings.direction = match cfg.settings.direction {
                    Direction::FrenchToEnglish => Direction::EnglishToFrench,
                    Direction::EnglishToFrench => Direction::FrenchToEnglish,
                };
            }
            'k' => cfg.settings.streaks_enabled = !cfg.settings.streaks_enabled,
            'u' => update_settings_menu(cfg, ui),
            'q' => {
                let _ = cfg.save();
                return;
            }
            _ => {}
        }
        let _ = cfg.save();
    }
}

fn direction_label(d: Direction) -> &'static str {
    match d {
        Direction::FrenchToEnglish => "French -> English",
        Direction::EnglishToFrench => "English -> French",
    }
}

fn on_off(b: bool) -> &'static str {
    if b {
        "on"
    } else {
        "off"
    }
}

fn update_settings_menu(cfg: &mut Config, ui: &mut Ui) {
    loop {
        ui.header("Update settings");
        ui.dim(&format!(
            "  always-update: {} | ask-before-update: {}",
            on_off(cfg.settings.always_update),
            on_off(cfg.settings.ask_before_update),
        ));
        println!();
        match ui.menu(&[
            ('a', "Toggle always update on launch"),
            ('b', "Toggle ask before updating"),
            ('n', "Run update now"),
            ('q', "Back"),
        ]) {
            'a' => cfg.settings.always_update = !cfg.settings.always_update,
            'b' => cfg.settings.ask_before_update = !cfg.settings.ask_before_update,
            'n' => run_update_ui(ui),
            'q' => return,
            _ => {}
        }
    }
}

pub fn run_update_ui(ui: &Ui) {
    ui.header("Updating");
    ui.dim("  Running 'git pull' in the install directory...\n");
    match update::run_update() {
        update::UpdateResult::UpdatedOrCurrent(log) => {
            if log.is_empty() {
                ui.good("  Already up to date.");
            } else {
                ui.say(&format!("  {}", log));
            }
        }
        update::UpdateResult::NotAGitCheckout => {
            ui.bad("  This install isn't a git checkout, so it can't self-update.");
            ui.dim("  Re-run install.sh / install.bat for the latest release, or");
            ui.dim("  on Nix: `nix flake update` in the source checkout, then rebuild.");
        }
        update::UpdateResult::GitMissing => {
            ui.bad("  Couldn't run git. Is it installed and on your PATH?");
        }
    }
    ui.pause();
}

pub fn choose_vocab_files(ui: &Ui, dir: &PathBuf) -> Vec<PathBuf> {
    let files = vocab::list_files(dir).unwrap_or_default();
    if files.is_empty() {
        return files;
    }
    let (topics, _other) = vocab::group_by_topic(&files);

    ui.header("Choose vocab files");

    if !topics.is_empty() {
        ui.accent("  Topics:");
        for (n, topic_files) in &topics {
            let names: Vec<String> = topic_files
                .iter()
                .map(|f| f.file_name().unwrap_or_default().to_string_lossy().to_string())
                .collect();
            ui.say(&format!(
                "      t{}) Topic {}  ({})",
                n,
                n,
                names.join(", ")
            ));
        }
        println!();
    }

    ui.accent("  Individual files:");
    for (i, f) in files.iter().enumerate() {
        ui.say(&format!(
            "      {}) {}",
            i + 1,
            f.file_name().unwrap_or_default().to_string_lossy()
        ));
    }

    ui.dim("\n  Enter topic(s) like 't1', file numbers, 'a' for all, or Enter for all.");
    let raw = ui.read_line("  > ");
    if raw.trim().is_empty() || raw.trim().eq_ignore_ascii_case("a") {
        return files;
    }

    let mut chosen = Vec::new();
    for tok in raw.split_whitespace() {
        if let Some(topic_str) = tok.strip_prefix(['t', 'T']) {
            if let Ok(n) = topic_str.parse::<u32>() {
                if let Some((_, topic_files)) = topics.iter().find(|(t, _)| *t == n) {
                    for f in topic_files {
                        if !chosen.contains(f) {
                            chosen.push(f.clone());
                        }
                    }
                }
            }
            continue;
        }
        if let Ok(n) = tok.parse::<usize>() {
            if n >= 1 && n <= files.len() && !chosen.contains(&files[n - 1]) {
                chosen.push(files[n - 1].clone());
            }
        }
    }
    if chosen.is_empty() {
        files
    } else {
        chosen
    }
}
