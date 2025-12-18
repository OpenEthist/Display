use iced::Element;
use iced::Point;
use iced::Rectangle;
use iced::time::{self};
use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path, Stroke};
use std::time::Duration;

pub enum UnderlineMessage {
    Tick,
}

pub struct Underline {
    pub width: f32,
    pub height: f32,
    /// "Framerate" delta. 16666 for 60 FPS.
    /// Use `calculate_delta` for generation.
    pub update_time: u32,
    phase: f32,
}

impl Underline {
    pub fn new() -> Self {
        Self {
            width: 600.0,
            height: 30.0,
            update_time: calculate_delta(60),
            phase: 0.0,
        }
    }

    pub fn view(&self) -> Element<'_, UnderlineMessage> {
        Canvas::new(self)
            .width(self.width)
            .height(self.height)
            .into()
    }

    pub fn update(&mut self, message: UnderlineMessage) {
        match message {
            UnderlineMessage::Tick => {
                let delta = 0.001875 * (self.update_time as f32 / 1000.0);
                self.phase += delta;
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<UnderlineMessage> {
        time::every(Duration::from_micros(self.update_time.into())).map(|_| UnderlineMessage::Tick)
    }
}

/// Framerate to microseconds deltas
const fn calculate_delta(framerate: u32) -> u32 {
    1000000 / framerate
}

// AI Generated
fn squiggle_path(bounds: Rectangle, amplitude: f32, wavelength: f32, phase: f32) -> Path {
    Path::new(|builder| {
        let y = bounds.height / 2.0;
        let start_x = 0.0;
        let end_x = bounds.width;

        builder.move_to(Point::new(start_x, y));

        let mut x = start_x;
        while x <= end_x {
            let offset = (x / wavelength + phase).sin() * amplitude;
            builder.line_to(Point::new(x, y + offset));
            x += 1.0;
        }
    })
}

// AI Generated
impl canvas::Program<UnderlineMessage> for Underline {
    type State = Cache;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let path = squiggle_path(
            bounds,
            // - 5.0 to prevent clipping. This is roughly double width
            self.height / 2.0 - 5.0, // amplitude
            8.0,                     // wavelength
            self.phase,
        );

        frame.stroke(
            &path,
            Stroke {
                width: 2.5,
                style: canvas::Style::Solid(theme.palette().primary),
                ..Stroke::default()
            },
        );

        vec![frame.into_geometry()]
    }
}
