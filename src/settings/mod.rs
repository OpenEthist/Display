use std::sync::LazyLock;

pub static SETTINGS: LazyLock<Settings> = LazyLock::new(Settings::default);

#[cfg(feature = "embedded")]
pub struct WifiSettings {
    ssid: String,
    psk: String,
}

#[derive(Default)]
pub struct Settings {
    #[cfg(feature = "embedded")]
    pub wifi: Option<WifiSettings>,
    #[cfg(feature = "embedded")]
    pub timezone: Option<String>,
    pub location: Option<(f32, f32)>,
    pub clock: ClockSettings,
    pub theme: ThemeSettings,
}

pub struct ThemeSettings {
    pub font: String,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            font: "Roboto".into(),
        }
    }
}

#[derive(Default)]
pub struct ClockSettings {
    pub seconds: bool,
    pub hour_24: bool,
}
