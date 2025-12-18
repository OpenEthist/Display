use iced::{
    Alignment, Element, Fill, Subscription,
    widget::{column, row},
};

use crate::components::{
    clock::{Clock, ClockMessage},
    line::{Underline, UnderlineMessage},
};

pub struct ClockScreen {
    clock: Clock,
    underline: Underline,
}

pub enum ClockScreenMessage {
    Clock(ClockMessage),
    Underline(UnderlineMessage),
}

impl ClockScreen {
    pub fn new() -> Self {
        Self {
            clock: Clock::new(),
            underline: Underline::new(),
        }
    }

    pub fn update(&mut self, message: ClockScreenMessage) {
        match message {
            ClockScreenMessage::Clock(msg) => self.clock.update(msg),
            ClockScreenMessage::Underline(msg) => self.underline.update(msg),
        }
    }

    pub fn view(&self) -> Element<'_, ClockScreenMessage> {
        row![
            column![
                self.clock.view().map(ClockScreenMessage::Clock),
                self.underline.view().map(ClockScreenMessage::Underline),
            ]
            .width(Fill)
            .align_x(Alignment::Center),
        ]
        .height(Fill)
        .align_y(Alignment::Center)
        .into()
    }

    pub fn subscription(&self) -> Subscription<ClockScreenMessage> {
        Subscription::batch([
            self.clock.subscription().map(ClockScreenMessage::Clock),
            self.underline
                .subscription()
                .map(ClockScreenMessage::Underline),
        ])
    }
}
