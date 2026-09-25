use crate::answer::validate_answer;
use crate::config::{Direction, Settings};
use crate::ui::Ui;
use crate::vocab::Pair;
use rand::seq::SliceRandom;
use rand::Rng;
use std::time::{Duration, Instant};

pub enum TypingMode {
    Continuous,
    Count(usize),
    Max,
    Timed(Duration),
}

pub enum McMode {
    Continuous,
    Count(usize),
    Max,
}

fn qa(pair: &Pair, direction: Direction) -> (String, String) {
    match direction {
        Direction::FrenchToEnglish => (pair.french.clone(), pair.english.clone()),
        Direction::EnglishToFrench => (pair.english.clone(), pair.french.clone()),
    }
}

struct Score {
    correct: u32,
    asked: u32,
}
impl Score {
    fn new() -> Self {
        Score {
            correct: 0,
            asked: 0,
        }
    }
    fn record(&mut self, correct: bool) {
        self.asked += 1;
        if correct {
            self.correct += 1;
        }
    }
    fn line(&self) -> String {
        format!("Score: {}/{}", self.correct, self.asked)
    }
}

fn ask_typing_question(ui: &Ui, settings: &Settings, pair: &Pair, score: &mut Score) -> bool {
    let (question, answer) = qa(pair, settings.direction);
    ui.accent(&format!("\n  Translate: {}", question));
    ui.dim("  (leave blank + Enter to stop)");
    let user = ui.read_line("  > ");
    if user.is_empty() {
        return false;
    }

    let (accepted, fuzzy) = validate_answer(&user, &answer, settings);
    score.record(accepted);
    if accepted {
        ui.good("  Correct!");
        if let Some(f) = fuzzy {
            ui.dim(&format!("  (accepted via fuzzy match, {}% similar)", f));
        }
    } else {
        ui.bad(&format!("  Incorrect. Correct answer: {}", answer));
    }
    if settings.score_enabled {
        ui.dim(&format!("  {}", score.line()));
    }
    true
}

pub fn run_typing(vocab: &[Pair], settings: &Settings, ui: &Ui, mode: TypingMode) {
    if vocab.len() < 2 {
        ui.bad("Not enough vocabulary loaded.");
        return;
    }
    let mut score = Score::new();
    let mut rng = rand::thread_rng();

    match mode {
        TypingMode::Continuous => {
            ui.dim("  Continuous mode.\n");
            loop {
                let pair = &vocab[rng.gen_range(0..vocab.len())];
                if !ask_typing_question(ui, settings, pair, &mut score) {
                    break;
                }
            }
        }
        TypingMode::Count(n) => {
            for i in 0..n {
                ui.dim(&format!("\n  Question {}/{}", i + 1, n));
                let pair = &vocab[rng.gen_range(0..vocab.len())];
                if !ask_typing_question(ui, settings, pair, &mut score) {
                    break;
                }
            }
        }
        TypingMode::Max => {
            let mut shuffled: Vec<&Pair> = vocab.iter().collect();
            shuffled.shuffle(&mut rng);
            let total = shuffled.len();
            for (i, pair) in shuffled.into_iter().enumerate() {
                ui.dim(&format!("\n  Question {}/{}", i + 1, total));
                if !ask_typing_question(ui, settings, pair, &mut score) {
                    break;
                }
            }
        }
        TypingMode::Timed(duration) => {
            ui.dim("  Timed quiz. Answer as many as you can!\n");
            let start = Instant::now();
            while start.elapsed() < duration {
                let remaining = duration - start.elapsed();
                ui.dim(&format!(
                    "  Time left: {}m {:02}s",
                    remaining.as_secs() / 60,
                    remaining.as_secs() % 60
                ));
                let pair = &vocab[rng.gen_range(0..vocab.len())];
                if !ask_typing_question(ui, settings, pair, &mut score) {
                    break;
                }
            }
            ui.accent("\n  Time's up!");
        }
    }
    ui.header("Quiz complete");
    ui.say(&format!("  Final {}", score.line()));
    ui.pause();
}

fn build_choices<'a>(
    vocab: &'a [Pair],
    settings: &Settings,
    correct: &Pair,
    rng: &mut impl Rng,
) -> Vec<String> {
    // Pick 3 random wrong answers, shuffle in correct
    let (_, correct_answer) = qa(correct, settings.direction);
    let mut pool: Vec<String> = vocab
        .iter()
        .map(|p| qa(p, settings.direction).1)
        .filter(|a| a != &correct_answer)
        .collect();
    pool.shuffle(rng);
    pool.dedup();
    let mut options: Vec<String> = pool.into_iter().take(3).collect();
    options.push(correct_answer);
    options.shuffle(rng);
    options
}

fn ask_mc_question(
    ui: &Ui,
    settings: &Settings,
    vocab: &[Pair],
    pair: &Pair,
    score: &mut Score,
) -> bool {
    let mut rng = rand::thread_rng();
    let (question, correct_answer) = qa(pair, settings.direction);
    let options = build_choices(vocab, settings, pair, &mut rng);

    ui.accent(&format!("\n  What is the translation of '{}'?\n", question));
    for (i, opt) in options.iter().enumerate() {
        ui.say(&format!("      {}) {}", i + 1, opt));
    }
    ui.dim("  (q to stop)");
    let raw = ui.read_line("\n  > ");
    if raw.trim().eq_ignore_ascii_case("q") {
        return false;
    }
    let picked: Option<usize> = raw.trim().parse::<usize>().ok();

    let correct = picked
        .and_then(|n| options.get(n.wrapping_sub(1)))
        .map(|s| s == &correct_answer)
        .unwrap_or(false);

    score.record(correct);
    if correct {
        ui.good("  Correct!");
    } else {
        ui.bad(&format!("  Incorrect. The answer was: {}", correct_answer));
    }
    if settings.score_enabled {
        ui.dim(&format!("  {}", score.line()));
    }
    true
}

pub fn run_multichoice(vocab: &[Pair], settings: &Settings, ui: &Ui, mode: McMode) {
    if vocab.len() < 4 {
        ui.bad("Need at least 4 vocab pairs for multiple choice.");
        return;
    }
    let mut score = Score::new();
    let mut rng = rand::thread_rng();

    match mode {
        McMode::Continuous => {
            ui.dim("  Continuous mode.\n");
            loop {
                let pair = &vocab[rng.gen_range(0..vocab.len())];
                if !ask_mc_question(ui, settings, vocab, pair, &mut score) {
                    break;
                }
            }
        }
        McMode::Count(n) => {
            for i in 0..n {
                ui.dim(&format!("\n  Question {}/{}", i + 1, n));
                let pair = &vocab[rng.gen_range(0..vocab.len())];
                if !ask_mc_question(ui, settings, vocab, pair, &mut score) {
                    break;
                }
            }
        }
        McMode::Max => {
            let mut shuffled: Vec<&Pair> = vocab.iter().collect();
            shuffled.shuffle(&mut rng);
            let total = shuffled.len();
            for (i, pair) in shuffled.into_iter().enumerate() {
                ui.dim(&format!("\n  Question {}/{}", i + 1, total));
                if !ask_mc_question(ui, settings, vocab, pair, &mut score) {
                    break;
                }
            }
        }
    }
    ui.header("Quiz complete");
    ui.say(&format!("  Final {}", score.line()));
    ui.pause();
}
