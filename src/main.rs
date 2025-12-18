mod components;
mod http_cache;
mod music;
mod screens;
mod settings;
mod theme;
mod spotify;

use iced::{Element, Subscription};

use music::{Music, MusicMessage};
use screens::clock::{ClockScreen, ClockScreenMessage};
use theme::DEFAULT_THEME;
use settings::Settings;

enum Screen {
    Clock,
    Music,
}

struct App {
    clock: ClockScreen,
    music: Music,

    settings: Settings,
    screen: Screen,
}

pub enum Message {
    Clock(ClockScreenMessage),
    Music(MusicMessage),
}

impl App {
    fn new() -> Self {
        Self {
            clock: ClockScreen::new(),
            music: Music::new(),
            settings: Settings::default(),

            screen: Screen::Clock,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Clock(msg) => self.clock.update(msg),
            Message::Music(msg) => self.music.update(msg),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        if self.music.song.is_some() {
            return self.music.view().map(Message::Music);
        }

        self.clock.view().map(Message::Clock)
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.clock.subscription().map(Message::Clock),
            self.music.spotify_subscription().map(Message::Music),
        ])
    }
}

fn main() {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .antialiasing(true)
        .decorations(false)
        .theme(DEFAULT_THEME)
        .run()
        .unwrap();
}
