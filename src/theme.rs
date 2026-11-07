//! ROCKET's UI theme system — five hand-tuned color palettes plus the
//! styled-widget helpers (cards, chips, solid buttons, text inputs) that
//! make the app look designed instead of default.

use iced::{widget::button as iced_button, widget::container, widget::text_input, Background, Border, Color, Theme};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiTheme {
    pub name: &'static str,
    /// Window background.
    pub bg: Color,
    /// Cards / panels.
    pub surface: Color,
    /// Secondary surfaces (AI bubbles, inputs).
    pub surface_alt: Color,
    pub border: Color,
    pub text: Color,
    pub text_dim: Color,
    /// Primary accent (buttons, user bubbles).
    pub accent: Color,
    pub accent_hover: Color,
    /// Secondary highlight (links, KB titles, rocket glow).
    pub highlight: Color,
}

fn rgb(hex: u32) -> Color {
    Color {
        r: ((hex >> 16) & 0xFF) as f32 / 255.0,
        g: ((hex >> 8) & 0xFF) as f32 / 255.0,
        b: (hex & 0xFF) as f32 / 255.0,
        a: 1.0,
    }
}

/// Color with a modified alpha channel (for tinted chips over surfaces).
pub fn alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

pub fn themes() -> [UiTheme; 5] {
    [
        UiTheme {
            name: "🚀 Rocket Night",
            bg: rgb(0x0b1020),
            surface: rgb(0x141b30),
            surface_alt: rgb(0x1b2440),
            border: rgb(0x2a3558),
            text: rgb(0xe9eefb),
            text_dim: rgb(0x8a94b8),
            accent: rgb(0xff7a18),
            accent_hover: rgb(0xff9440),
            highlight: rgb(0x7cf9ff),
        },
        UiTheme {
            name: "🌌 Nebula",
            bg: rgb(0x120e22),
            surface: rgb(0x1c1633),
            surface_alt: rgb(0x251d44),
            border: rgb(0x372a63),
            text: rgb(0xf0ecff),
            text_dim: rgb(0x9c93c4),
            accent: rgb(0xa78bfa),
            accent_hover: rgb(0xbda5ff),
            highlight: rgb(0xf472b6),
        },
        UiTheme {
            name: "🌿 Forest",
            bg: rgb(0x0b1410),
            surface: rgb(0x132019),
            surface_alt: rgb(0x1a2b21),
            border: rgb(0x27402f),
            text: rgb(0xe7f5ec),
            text_dim: rgb(0x86a892),
            accent: rgb(0x34d399),
            accent_hover: rgb(0x5eeab8),
            highlight: rgb(0xa3e635),
        },
        UiTheme {
            name: "🔥 Inferno",
            bg: rgb(0x170c08),
            surface: rgb(0x241310),
            surface_alt: rgb(0x311a14),
            border: rgb(0x4a2a1e),
            text: rgb(0xfbeee6),
            text_dim: rgb(0xbd9a86),
            accent: rgb(0xf97316),
            accent_hover: rgb(0xfb923c),
            highlight: rgb(0xfbbf24),
        },
        UiTheme {
            name: "❄️ Frost",
            bg: rgb(0xecf1f8),
            surface: rgb(0xffffff),
            surface_alt: rgb(0xf2f6fb),
            border: rgb(0xd3dce8),
            text: rgb(0x16213a),
            text_dim: rgb(0x5b6a85),
            accent: rgb(0x2563eb),
            accent_hover: rgb(0x4f82f5),
            highlight: rgb(0x0891b2),
        },
    ]
}

/// The palette for a saved index (clamped).
pub fn palette(idx: usize) -> UiTheme {
    let themes = themes();
    themes[idx.min(themes.len() - 1)]
}

// ---------------------------------------------------------------- persistence

const THEME_FILE: &str = "rocket_theme.json";

/// Load the saved theme index (falls back to 0).
pub fn load_index() -> usize {
    std::fs::read_to_string(THEME_FILE)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v["theme"].as_u64())
        .map(|n| (n as usize).min(themes().len() - 1))
        .unwrap_or(0)
}

/// Persist the theme index.
pub fn save_index(idx: usize) {
    let idx = idx.min(themes().len() - 1);
    let _ = std::fs::write(
        THEME_FILE,
        serde_json::json!({ "theme": idx }).to_string(),
    );
}

// ------------------------------------------------------------- style helpers

fn border(radius: f32) -> Border {
    Border {
        radius: radius.into(),
        ..Default::default()
    }
}

/// Rounded card container with border.
pub fn card(th: UiTheme) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        text_color: Some(th.text),
        background: Some(Background::Color(th.surface)),
        border: {
            let mut b = border(10.0);
            b.color = th.border;
            b.width = 1.0;
            b
        },
        ..Default::default()
    }
}

/// Flat container (bubbles) with a background and optional border.
pub fn flat(bg: Color, edge: Color, radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        text_color: None,
        background: Some(Background::Color(bg)),
        border: {
            let mut b = border(radius);
            b.color = edge;
            b.width = 1.0;
            b
        },
        ..Default::default()
    }
}

/// Chip button: tinted surface, 1px border, hover brightens.
pub fn chip(bg: Color, edge: Color, text_color: Color, hover: Color) -> impl Fn(&Theme, iced_button::Status) -> iced_button::Style {
    move |_t, status| {
        let background = match status {
            iced_button::Status::Hovered | iced_button::Status::Pressed => hover,
            _ => bg,
        };
        iced_button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: {
                let mut b = border(8.0);
                b.color = edge;
                b.width = 1.0;
                b
            },
            ..Default::default()
        }
    }
}

/// Solid accent button, white text, rounded.
pub fn solid(base: Color, hover: Color) -> impl Fn(&Theme, iced_button::Status) -> iced_button::Style {
    move |_t, status| {
        let background = match status {
            iced_button::Status::Hovered => hover,
            iced_button::Status::Pressed => hover,
            iced_button::Status::Disabled => base,
            iced_button::Status::Active => base,
        };
        iced_button::Style {
            background: Some(Background::Color(background)),
            text_color: Color::WHITE,
            border: border(8.0),
            ..Default::default()
        }
    }
}

/// Themed text input.
pub fn input(th: UiTheme) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_t, _status| text_input::Style {
        background: Background::Color(th.surface_alt),
        border: {
            let mut b = border(8.0);
            b.color = th.border;
            b.width = 1.0;
            b
        },
        icon: th.text_dim,
        placeholder: th.text_dim,
        value: th.text,
        selection: th.accent,
    }
}

/// Banner styling for headers.
pub fn banner(bg: Color, text: Color) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        text_color: Some(text),
        background: Some(Background::Color(bg)),
        border: border(12.0),
        ..Default::default()
    }
}

/// Chat message bubble styling.
pub fn chat_message(bg: Color, text: Color) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        text_color: Some(text),
        background: Some(Background::Color(bg)),
        border: border(16.0),
        ..Default::default()
    }
}

/// Basic button styling.
pub fn button<'a, Message>(base: Color) -> impl Fn(&Theme, iced_button::Status) -> iced_button::Style + 'a {
    move |_t, status| {
        let background = match status {
            iced_button::Status::Hovered | iced_button::Status::Pressed => base,
            _ => base,
        };
        iced_button::Style {
            background: Some(Background::Color(background)),
            text_color: Color::WHITE,
            border: border(8.0),
            ..Default::default()
        }
    }
}
