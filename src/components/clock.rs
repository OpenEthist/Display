use chrono::Timelike;
use iced::{
    Element, Subscription,
    font::Weight,
    time::{self, milliseconds},
    widget::{Row, Text},
};

use crate::settings::SETTINGS;
use crate::theme::get_font;

pub struct Clock {
    now: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Clone, Copy)]
pub enum ClockMessage {
    Tick(chrono::DateTime<chrono::Local>),
}

fn text<'a>(str: impl iced::widget::text::IntoFragment<'a>) -> Element<'a, ClockMessage> {
    let font_size = 100;
    let mut font = get_font();
    font.weight = Weight::Bold;

    Text::new(str).font(font).size(font_size).into()
}

impl Clock {
    pub fn new() -> Self {
        Self {
            now: chrono::offset::Local::now(),
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
        let mut result = Row::new();
        let show_seconds = SETTINGS.clock.seconds;
        let hide_meridiem = SETTINGS.clock.hour_24;

        // Hour (and meridiem)
        let (meridiem, hour) = if hide_meridiem {
            let hour = self.now.hour();
            (false, hour)
        } else {
            self.now.hour12()
        };

        // Minute
        let minute = self.now.minute();
        let minute_text = format!("{minute:0>2}");
        result = result.extend([text(hour), text(":"), text(minute_text)]);

        // Seconds
        if show_seconds {
            let second = self.now.second();
            result = result.extend([text(":"), text(second)])
        }

        // AM/PM
        if !hide_meridiem {
            let meridiem_text = if meridiem { text(" PM") } else { text(" AM") };
            result = result.extend([meridiem_text]);
        }

        result.into()
    }

    pub fn subscription(&self) -> Subscription<ClockMessage> {
        // Just under a secondintentionally
        // Stops rare "double second" change.
        let time = milliseconds(950);

        time::every(time).map(|_| ClockMessage::Tick(chrono::offset::Local::now()))
    }
}
