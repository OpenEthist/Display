use iced::{Font, Theme};

use crate::settings::SETTINGS;

pub const DEFAULT_THEME: Theme = Theme::CatppuccinMocha;
const DEFAULT_FONT: &str = "Roboto";

pub fn get_font() -> Font {
    Font::with_name(&SETTINGS.theme.font)
}
