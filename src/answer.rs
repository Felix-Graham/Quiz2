use crate::config::Settings;

const ARTICLES: &[&str] = &["the ", "a ", "an ", "to be ", "to "];

fn strip_articles(mut s: String) -> String {
    for art in ARTICLES {
        if let Some(stripped) = s.strip_prefix(art) {
            s = stripped.to_string();
        }
    }
    s
}

fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}

pub fn validate_answer(user: &str, correct: &str, settings: &Settings) -> (bool, Option<u8>) {
    let user_n = normalize(user);
    let correct_n = normalize(correct);

    if user_n == correct_n {
        return (true, None);
    }

    if !settings.wide_range_enabled {
        return (false, None);
    }

    let user_w = strip_articles(user_n);
    let correct_w = strip_articles(correct_n);
    if user_w == correct_w {
        return (true, None);
    }

    let similarity = strsim::normalized_levenshtein(&user_w, &correct_w) * 100.0;
    let score = similarity.round().clamp(0.0, 100.0) as u8;
    (score >= settings.fuzzy_threshold, Some(score))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Direction, Theme};

    fn settings(wide: bool) -> Settings {
        Settings {
            score_enabled: true,
            wide_range_enabled: wide,
            fuzzy_threshold: 80,
            theme: Theme::Dark,
            direction: Direction::FrenchToEnglish,
            always_update: false,
            ask_before_update: true,
            streaks_enabled: true,
        }
    }

    #[test]
    fn exact_match() {
        assert_eq!(validate_answer("Hello", "hello", &settings(false)).0, true);
    }

    #[test]
    fn article_widening() {
        assert_eq!(validate_answer("cat", "the cat", &settings(true)).0, true);
        assert_eq!(validate_answer("cat", "the cat", &settings(false)).0, false);
    }

    #[test]
    fn fuzzy_typo() {
        let (ok, score) = validate_answer("helo", "hello", &settings(true));
        assert!(ok);
        assert!(score.is_some());
    }

    #[test]
    fn fuzzy_off_rejects_typo() {
        assert_eq!(validate_answer("helo", "hello", &settings(false)).0, false);
    }

    #[test]
    fn wrong_answer_rejected() {
        assert_eq!(validate_answer("dog", "cat", &settings(true)).0, false);
    }
}
