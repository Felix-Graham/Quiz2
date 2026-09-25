use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Pair {
    pub french: String,
    pub english: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Delimited,
    Legacy,
}

pub struct ParseReport {
    pub pairs: Vec<Pair>,
    pub format: Format,
    pub total_lines: usize,
    pub blank_lines: usize,
    pub header_lines: usize,
    pub content_lines: usize,
    pub structurally_valid: bool,
}

pub fn list_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().map(|e| e == "txt").unwrap_or(false)
                && !p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .contains('~')
        })
        .collect();
    files.sort();
    Ok(files)
}

/// Extracts the topic number from a `vocab_<topic>-<sub>.txt` style name.
/// Anything else (e.g. `essay_vocab.txt`) returns `None` and is treated as
/// a standalone file rather than part of a topic.
pub fn topic_number(path: &Path) -> Option<u32> {
    let stem = path.file_stem()?.to_str()?.to_lowercase();
    let rest = stem
        .strip_prefix("vocab_")
        .or_else(|| stem.strip_prefix("vocab-"))?;

    let topic_digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if topic_digits.is_empty() {
        return None;
    }
    let after_topic = &rest[topic_digits.len()..];
    let mut chars = after_topic.chars();
    match chars.next() {
        Some('-') | Some('_') => {}
        _ => return None,
    }
    let sub_digits: String = chars.take_while(|c| c.is_ascii_digit()).collect();
    if sub_digits.is_empty() {
        return None;
    }
    topic_digits.parse().ok()
}

/// Groups vocab files by topic number, preserving filename order within
/// each topic. Files that don't match the `vocab_<topic>-<sub>.txt`
/// pattern come back separately, untouched.
pub fn group_by_topic(files: &[PathBuf]) -> (Vec<(u32, Vec<PathBuf>)>, Vec<PathBuf>) {
    let mut topics: Vec<(u32, Vec<PathBuf>)> = Vec::new();
    let mut other = Vec::new();

    for f in files {
        match topic_number(f) {
            Some(n) => match topics.iter_mut().find(|(t, _)| *t == n) {
                Some((_, list)) => list.push(f.clone()),
                None => topics.push((n, vec![f.clone()])),
            },
            None => other.push(f.clone()),
        }
    }
    topics.sort_by_key(|(n, _)| *n);
    (topics, other)
}

// Removes "1.", "2)", "-", "*", "•" list markers some editors add.
fn strip_marker(line: &str) -> &str {
    let trimmed = line.trim_start();
    let after_bullet = trimmed
        .strip_prefix('-')
        .or_else(|| trimmed.strip_prefix('*'))
        .or_else(|| trimmed.strip_prefix('•'))
        .map(|s| s.trim_start());
    if let Some(rest) = after_bullet {
        return rest;
    }
    let digits_end = trimmed
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_digit())
        .last()
        .map(|(i, c)| i + c.len_utf8());
    if let Some(end) = digits_end {
        if end > 0 {
            let rest = &trimmed[end..];
            if let Some(rest) = rest.strip_prefix('.').or_else(|| rest.strip_prefix(')')) {
                return rest.trim_start();
            }
        }
    }
    trimmed
}

pub fn parse_file(path: &Path) -> std::io::Result<Vec<Pair>> {
    Ok(parse_file_report(path)?.pairs)
}

pub fn parse_file_report(path: &Path) -> std::io::Result<ParseReport> {
    let content = fs::read_to_string(path)?;
    let raw_lines: Vec<&str> = content.lines().collect();
    let total_lines = raw_lines.len();

    let uses_delimiter = raw_lines.iter().any(|l| l.contains('|'));

    if uses_delimiter {
        let mut pairs = Vec::new();
        let mut blank_lines = 0;
        let mut header_lines = 0;
        let mut content_lines = 0;

        for line in &raw_lines {
            let line = strip_marker(line.trim());
            if line.is_empty() {
                blank_lines += 1;
                continue;
            }
            if line.starts_with('#') {
                header_lines += 1;
                continue;
            }
            content_lines += 1;
            if let Some((fr, en)) = line.split_once('|') {
                let fr = fr.trim().to_string();
                let en = en.trim().to_string();
                if !fr.is_empty() && !en.is_empty() {
                    pairs.push(Pair {
                        french: fr,
                        english: en,
                    });
                }
            }
        }

        // Each line is a self-contained pair, so there's no f/e
        // alternation to break - "valid" just means every content line
        // actually parsed into a pair.
        let structurally_valid = content_lines == pairs.len();

        Ok(ParseReport {
            pairs,
            format: Format::Delimited,
            total_lines,
            blank_lines,
            header_lines,
            content_lines,
            structurally_valid,
        })
    } else {
        let blank_lines = raw_lines.iter().filter(|l| l.trim().is_empty()).count();
        let mut body: Vec<String> = raw_lines
            .iter()
            .map(|l| strip_marker(l.trim_end()).to_string())
            .filter(|l| !l.trim().is_empty())
            .collect();
        let non_blank = body.len();

        // A lone odd line out is assumed to be a title header, since a
        // well-formed f>e>f>e... file always has an even number of
        // content lines.
        let header_lines = if non_blank % 2 == 1 { 1 } else { 0 };
        if header_lines == 1 {
            body.remove(0);
        }
        let content_lines = body.len();
        let structurally_valid = content_lines % 2 == 0;

        let mut pairs = Vec::new();
        let mut it = body.into_iter();
        while let (Some(fr), Some(en)) = (it.next(), it.next()) {
            pairs.push(Pair {
                french: fr,
                english: en,
            });
        }

        Ok(ParseReport {
            pairs,
            format: Format::Legacy,
            total_lines,
            blank_lines,
            header_lines,
            content_lines,
            structurally_valid,
        })
    }
}

pub fn merge(files: &[PathBuf]) -> Vec<Pair> {
    let mut total = Vec::new();
    for f in files {
        if let Ok(mut pairs) = parse_file(f) {
            total.append(&mut pairs);
        }
    }
    total
}

pub fn write_starter_file(dir: &Path) -> std::io::Result<()> {
    let path = dir.join("01-basics.txt");
    if path.exists() {
        return Ok(());
    }
    let starter = "\
# French Vocab Quiz starter file. One pair per line: french | english
bonjour | hello
au revoir | goodbye
merci | thank you
s'il vous plaît | please
oui | yes
non | no
le chat | the cat
le chien | the dog
la maison | the house
manger | to eat
";
    fs::write(path, starter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_topic_pattern() {
        assert_eq!(topic_number(Path::new("vocab_1-1.txt")), Some(1));
        assert_eq!(topic_number(Path::new("vocab_4-2.txt")), Some(4));
        assert_eq!(topic_number(Path::new("VOCAB_10-3.txt")), Some(10));
    }

    #[test]
    fn rejects_unstructured_names() {
        assert_eq!(topic_number(Path::new("essay_vocab.txt")), None);
        assert_eq!(topic_number(Path::new("vocab.txt")), None);
        assert_eq!(topic_number(Path::new("vocab_1.txt")), None);
        assert_eq!(topic_number(Path::new("random_notes.txt")), None);
    }

    #[test]
    fn groups_by_topic() {
        let files = vec![
            PathBuf::from("vocab_1-1.txt"),
            PathBuf::from("vocab_1-2.txt"),
            PathBuf::from("vocab_2-1.txt"),
            PathBuf::from("essay_vocab.txt"),
        ];
        let (topics, other) = group_by_topic(&files);
        assert_eq!(topics.len(), 2);
        assert_eq!(topics[0].0, 1);
        assert_eq!(topics[0].1.len(), 2);
        assert_eq!(topics[1].0, 2);
        assert_eq!(other, vec![PathBuf::from("essay_vocab.txt")]);
    }
}
