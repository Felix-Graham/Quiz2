use crate::config::install_dir;
use std::process::Command;

pub enum UpdateResult {
    UpdatedOrCurrent(String),
    NotAGitCheckout,
    GitMissing,
}

pub fn run_update() -> UpdateResult {
    let dir = install_dir();
    if !dir.join(".git").exists() {
        return UpdateResult::NotAGitCheckout;
    }

    match Command::new("git").arg("pull").current_dir(&dir).output() {
        Ok(out) => {
            let mut text = String::from_utf8_lossy(&out.stdout).to_string();
            text.push_str(&String::from_utf8_lossy(&out.stderr));
            UpdateResult::UpdatedOrCurrent(text.trim().to_string())
        }
        Err(_) => UpdateResult::GitMissing,
    }
}
