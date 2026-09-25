use crate::config::Theme;
use crossterm::style::Color;

#[derive(Clone, Copy)]
pub struct Palette {
    pub fg: Color,
    pub dim: Color,
    pub accent: Color,
    pub good: Color,
    pub bad: Color,
    pub bg: Color,
}

pub fn palette(theme: Theme) -> Palette {
    match theme {
        Theme::Dark => Palette {
            fg: Color::Rgb {
                r: 220,
                g: 220,
                b: 210,
            },
            dim: Color::Rgb {
                r: 120,
                g: 120,
                b: 120,
            },
            accent: Color::Rgb {
                r: 130,
                g: 190,
                b: 255,
            },
            good: Color::Rgb {
                r: 130,
                g: 220,
                b: 140,
            },
            bad: Color::Rgb {
                r: 230,
                g: 110,
                b: 110,
            },
            bg: Color::Black,
        },
        Theme::Light => Palette {
            fg: Color::Rgb {
                r: 30,
                g: 30,
                b: 30,
            },
            dim: Color::Rgb {
                r: 110,
                g: 110,
                b: 110,
            },
            accent: Color::Rgb {
                r: 20,
                g: 90,
                b: 160,
            },
            good: Color::Rgb {
                r: 30,
                g: 130,
                b: 40,
            },
            bad: Color::Rgb {
                r: 170,
                g: 30,
                b: 30,
            },
            bg: Color::White,
        },
    }
}
