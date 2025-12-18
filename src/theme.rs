use iced::{Font, Theme};

pub const DEFAULT_THEME: Theme = Theme::CatppuccinMocha;
const DEFAULT_FONT: &str = "Roboto";

pub fn get_font() -> Font {
    Font::with_name(DEFAULT_FONT)
}
