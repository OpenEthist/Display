struct WifiSettings {
    ssid: String,
    psk: String,
}

struct Settings {
    wifi: WifiSettings,
    timezone: String,
    location: (f32, f32),
}

