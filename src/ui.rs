use crate::theme::{palette, Palette};
use crate::config::Theme;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{self, ClearType};
use crossterm::{execute, queue};
use std::io::{stdout, Write};

pub struct Ui {
    pub palette: Palette,
}

impl Ui {
    pub fn new(theme: Theme) -> Ui {
        Ui {
            palette: palette(theme),
        }
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.palette = palette(theme);
    }

    pub fn clear(&self) {
        let mut out = stdout();
        let _ = execute!(
            out,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        );
    }

    pub fn say(&self, text: &str) {
        self.color_line(text, self.palette.fg);
    }

    pub fn dim(&self, text: &str) {
        self.color_line(text, self.palette.dim);
    }

    pub fn accent(&self, text: &str) {
        self.color_line(text, self.palette.accent);
    }

    pub fn good(&self, text: &str) {
        self.color_line(text, self.palette.good);
    }

    pub fn bad(&self, text: &str) {
        self.color_line(text, self.palette.bad);
    }

    fn color_line(&self, text: &str, color: Color) {
        let mut out = stdout();
        let _ = queue!(out, SetForegroundColor(color), Print(text), Print("\n"), ResetColor);
        let _ = out.flush();
    }

    pub fn header(&self, title: &str) {
        self.clear();
        self.accent(&format!("  {}", title));
        self.dim(&format!("  {}", "-".repeat(40)));
        println!();
    }

    pub fn read_line(&self, prompt: &str) -> String {
        let mut out = stdout();
        let _ = queue!(out, SetForegroundColor(self.palette.accent), Print(prompt));
        let _ = out.flush();
        let mut buf = String::new();
        let _ = std::io::stdin().read_line(&mut buf);
        let _ = execute!(out, ResetColor);
        buf.trim().to_string()
    }

    pub fn read_key(&self) -> Option<char> {
        // Raw mode: read keypress without Enter
        let _ = terminal::enable_raw_mode();
        let result = loop {
            match event::read() {
                Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => match k.code {
                    KeyCode::Char('c')
                        if k.modifiers.contains(event::KeyModifiers::CONTROL) =>
                    {
                        break None;
                    }
                    KeyCode::Esc => break None,
                    KeyCode::Char(c) => break Some(c.to_ascii_lowercase()),
                    KeyCode::Enter => break Some('\n'),
                    _ => continue,
                },
                Ok(_) => continue,
                Err(_) => break None,
            }
        };
        let _ = terminal::disable_raw_mode();
        result
    }

    pub fn menu(&self, options: &[(char, &str)]) -> char {
        loop {
            for (key, label) in options {
                self.color_line(&format!("      {}  {}", key, label), self.palette.fg);
            }
            println!();
            if let Some(pressed) = self.read_key() {
                if options.iter().any(|(k, _)| *k == pressed) {
                    return pressed;
                }
            }
            self.clear();
            self.dim("      (unrecognised key, try again)\n");
        }
    }

    pub fn pause(&self) {
        self.dim("  -- press any key --");
        let _ = self.read_key();
    }
}
