mod answer;
mod config;
mod menu;
mod quiz;
mod streak;
mod theme;
mod ui;
mod update;
mod vocab;

use config::Config;
use std::path::PathBuf;
use std::time::Duration;
use ui::Ui;

fn main() {
    let mut cfg = Config::load();
    config::ensure_vocab_dir_exists(&config::vocab_dir()).ok();
    let _ = vocab::write_starter_file(&config::vocab_dir());

    let mut ui = Ui::new(cfg.settings.theme);

    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg = args.first().map(|s| s.as_str());

    match arg {
        Some("check") => {
            let dir = args.get(1).map(PathBuf::from).unwrap_or_else(config::vocab_dir);
            run_check(&dir);
            return;
        }
        Some("update") => {
            menu::run_update_ui(&ui);
            return;
        }
        Some("all") => {
            maybe_autoupdate(&cfg, &ui);
            let files = vocab::list_files(&config::vocab_dir()).unwrap_or_default();
            start_flow(&mut cfg, &mut ui, files);
            return;
        }
        Some("select") => {
            maybe_autoupdate(&cfg, &ui);
            let files = menu::choose_vocab_files(&ui, &config::vocab_dir());
            start_flow(&mut cfg, &mut ui, files);
            return;
        }
        Some(other) => {
            ui.bad(&format!("Unknown option '{other}'."));
            ui.say("Usage: frenchquiz [all|select|update|check]  (or run with no arguments for the menu)");
            return;
        }
        None => {}
    }

    maybe_autoupdate(&cfg, &ui);
    home_loop(&mut cfg, &mut ui);
}

fn run_check(dir: &PathBuf) {
    let files = match vocab::list_files(dir) {
        Ok(f) => f,
        Err(e) => {
            println!("Couldn't read {:?}: {}", dir, e);
            return;
        }
    };
    if files.is_empty() {
        println!("No .txt files found in {:?}", dir);
        return;
    }
    for f in files {
        let report = match vocab::parse_file_report(&f) {
            Ok(r) => r,
            Err(e) => {
                println!("== {:?}: error reading file ({}) ==", f.file_name().unwrap(), e);
                continue;
            }
        };
        println!("== {:?} ==", f.file_name().unwrap());
        match report.format {
            vocab::Format::Delimited => {
                println!(
                    "  format: french | english, one pair per line
  total lines: {}  |  blank/comment lines: {}  |  entry lines: {}
  parsed pairs: {}",
                    report.total_lines,
                    report.blank_lines + report.header_lines,
                    report.content_lines,
                    report.pairs.len(),
                );
                if report.structurally_valid {
                    println!("  check: every entry line parsed into a pair.");
                } else {
                    println!(
                        "  check: FAILED - {} entry line(s) didn't split on '|' correctly.",
                        report.content_lines - report.pairs.len()
                    );
                }
            }
            vocab::Format::Legacy => {
                let non_blank = report.total_lines - report.blank_lines;
                println!(
                    "  format: legacy alternating french/english lines
  total lines: {}  -  blank lines: {}  =  {} non-blank lines
  non-blank lines: {}  -  header lines: {}  =  {} lines to pair
  parsed pairs: {}",
                    report.total_lines,
                    report.blank_lines,
                    non_blank,
                    non_blank,
                    report.header_lines,
                    report.content_lines,
                    report.pairs.len(),
                );
                if report.structurally_valid {
                    println!(
                        "  check: {} is EVEN - likely a valid f>e>f>e... structure.",
                        report.content_lines
                    );
                } else {
                    println!(
                        "  check: FAILED - {} is ODD, so lines can't split evenly into",
                        report.content_lines
                    );
                    println!("  pairs. A translation may span two lines, or a line is missing.");
                }
                println!(
                    "  note: an even count is a good sign, not a guarantee - a single"
                );
                println!(
                    "  wrapped line still leaves an even total while shifting every pair."
                );
            }
        }
        for (i, p) in report.pairs.iter().enumerate() {
            println!("  {:>3}. {}  ->  {}", i + 1, p.french, p.english);
        }
        println!();
    }
}

fn maybe_autoupdate(cfg: &Config, ui: &Ui) {
    if cfg.settings.always_update {
        menu::run_update_ui(ui);
    }
}

fn home_loop(cfg: &mut Config, ui: &mut Ui) {
    loop {
        let streak_note = if cfg.settings.streaks_enabled {
            format!("{} day(s) (best {})", cfg.streak.current, cfg.streak.best)
        } else {
            "off".to_string()
        };

        ui.header("French Vocabulary Quiz");
        ui.dim(&format!("  streak: {}\n", streak_note));

        match ui.menu(&[
            ('s', "Start"),
            ('u', "Update"),
            ('i', "Settings"),
            ('q', "Quit"),
        ]) {
            's' => {
                let files = menu::choose_vocab_files(ui, &config::vocab_dir());
                start_flow(cfg, ui, files);
            }
            'u' => menu::run_update_ui(ui),
            'i' => menu::settings_menu(cfg, ui),
            'q' => {
                ui.clear();
                return;
            }
            _ => {}
        }
    }
}

fn start_flow(cfg: &mut Config, ui: &mut Ui, files: Vec<PathBuf>) {
    if files.is_empty() {
        ui.bad("No vocab files found. Add some .txt files under the 'vocab' folder.");
        ui.pause();
        return;
    }
    let vocab_list = vocab::merge(&files);
    if vocab_list.len() < 2 {
        ui.bad("Not enough vocabulary loaded from the selected files.");
        ui.pause();
        return;
    }

    let update = streak::record_practice(&mut cfg.streak, cfg.settings.streaks_enabled);
    let _ = cfg.save();
    announce_streak(ui, update, cfg.streak.current);

    ui.header("Select quiz type");
    let quiz_type = ui.menu(&[
        ('1', "Typing (type the translation)"),
        ('2', "Multiple choice"),
        ('q', "Cancel"),
    ]);
    if quiz_type == 'q' {
        return;
    }

    ui.header("Select mode");
    let mode_key = ui.menu(&[
        ('1', "Continuous"),
        ('2', "Set number of questions"),
        ('3', "Maximum (all questions)"),
        ('4', "Timed"),
        ('q', "Cancel"),
    ]);
    if mode_key == 'q' {
        return;
    }

    ui.clear();
    match quiz_type {
        '1' => {
            let mode = match mode_key {
                '1' => quiz::TypingMode::Continuous,
                '3' => quiz::TypingMode::Max,
                '4' => {
                    let mins: u64 = ui.read_line("  Minutes: ").parse().unwrap_or(0);
                    let secs: u64 = ui.read_line("  Seconds: ").parse().unwrap_or(0);
                    quiz::TypingMode::Timed(Duration::from_secs(mins * 60 + secs))
                }
                _ => {
                    let n: usize = ui
                        .read_line("  How many questions? ")
                        .parse()
                        .unwrap_or(10);
                    quiz::TypingMode::Count(n)
                }
            };
            quiz::run_typing(&vocab_list, &cfg.settings, ui, mode);
        }
        '2' => {
            let mode = match mode_key {
                '1' => quiz::McMode::Continuous,
                '3' => quiz::McMode::Max,
                _ => {
                    let n: usize = ui
                        .read_line("  How many questions? ")
                        .parse()
                        .unwrap_or(10);
                    quiz::McMode::Count(n)
                }
            };
            quiz::run_multichoice(&vocab_list, &cfg.settings, ui, mode);
        }
        _ => {}
    }
}

fn announce_streak(ui: &Ui, update: streak::StreakUpdate, current: u32) {
    use streak::StreakUpdate::*;
    match update {
        Incremented | StartedNew => {
            ui.good(&format!("  Streak: {} day(s)! Keep it up.\n", current));
        }
        Reset => {
            ui.bad("  Streak reset - you missed a day. Starting a new one today.\n");
        }
        AlreadyLoggedToday => {}
    }
}
