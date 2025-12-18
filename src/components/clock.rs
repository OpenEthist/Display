use chrono::Timelike;
use iced::Font;
use iced::font::Weight;
use iced::time::{self, milliseconds, seconds};
use iced::widget::text;
use iced::{Element, Subscription};

use crate::theme::get_font;

pub struct Clock {
    now: chrono::DateTime<chrono::Local>,
    seconds: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ClockMessage {
    Tick(chrono::DateTime<chrono::Local>),
}

impl Clock {
    pub fn new() -> Self {
        Self {
            now: chrono::offset::Local::now(),
            seconds: false,
        }
    }

    pub fn update(&mut self, message: ClockMessage) {
        match message {
            ClockMessage::Tick(local_time) => {
                if self.now != local_time {
                    self.now = local_time;
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, ClockMessage> {
        let (meridiem, hour) = self.now.hour12();
        let minute = self.now.minute();

        let meridiem_text = if meridiem { "PM" } else { "AM" };

        let mut font = get_font();
        font.weight = Weight::Bold;

        let text = text!("{hour}:{minute:0>2} {meridiem_text}")
            .font(font)
            .size(100);

        text.into()
    }

    pub fn subscription(&self) -> Subscription<ClockMessage> {
        // Just under a second/minute intentionally
        // Stops rare "double second/minute" change.
        let time = match self.seconds {
            true => milliseconds(950),
            false => seconds(59),
        };

        time::every(time).map(|_| ClockMessage::Tick(chrono::offset::Local::now()))
    }
}
