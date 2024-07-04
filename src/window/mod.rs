use crossterm::style::Color;
pub mod textbox;
pub mod textline;
pub mod window;

pub const GRUVBOX_BACKGROUND: Color = Color::Rgb {
    r: 40,
    g: 40,
    b: 40,
};

pub const GRUVBOX_FOCUS_BACKGROUND: Color = Color::Rgb {
    r: 50,
    g: 50,
    b: 50,
};
