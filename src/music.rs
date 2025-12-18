use iced::{
    Alignment, Background, Color, Element, Fill, Subscription,
    futures::{SinkExt, Stream, StreamExt, join},
    widget::{
        Container, Text, column, container,
        image::{Handle, Image},
        text,
    },
};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use crate::spotify::init_spotify;
use crate::theme::get_font;

const CACHE: &str = ".cache";
const CACHE_FILES: &str = ".cache/files";
// 100MB
const CACHE_SIZE: u64 = 1024 * 1024 * 100;

pub enum MusicMessage {
    UpdateSong(Song),
    RemoveSong,
    UpdateCover(PathBuf),
    UpdateColor(Color),
}

pub struct Music {
    pub song: Option<Song>,
}

pub struct Song {
    pub paused: bool,
    pub name: String,
    pub artist: String,
    pub length: u32,
    pub at: u32,
    pub cover: Option<Handle>,
    pub cover_url: String,
    pub color: Option<Color>,
}

impl Music {
    pub fn new() -> Self {
        Self { song: None }
    }

    pub fn view(&self) -> Element<'_, MusicMessage> {
        match &self.song {
            Some(song) => {
                let container = Container::new(Music::view_details(&song))
                    .height(Fill)
                    .width(Fill)
                    .align_y(Alignment::Center)
                    .align_x(Alignment::Center)
                    .style(|_theme| container::Style {
                        background: song.color.map(Background::Color),
                        ..Default::default()
                    });

                container.into()
            }
            None => text!("").into(),
        }
    }

    pub fn view_details(song: &Song) -> Element<'_, MusicMessage> {
        // Load the image
        let img: Element<'_, MusicMessage> = match &song.cover {
            Some(cover) => Image::new(cover)
                .height(180)
                .height(180)
                .border_radius(12.0)
                .into(),
            None => text!("loading...").into(),
        };

        column![
            // Pad the top so the cover is always centered
            column![Text::new("").size(32.0), Text::new("").size(18.0)].spacing(4),
            img,
            column![
                Text::new(&song.name).font(get_font()).size(28.0),
                Text::new(&song.artist).font(get_font()).size(18.0)
            ]
            .spacing(4)
        ]
        .padding(16.0)
        .spacing(16)
        .into()
    }

    pub fn update(&mut self, message: MusicMessage) {
        match message {
            MusicMessage::RemoveSong => self.song = None,
            MusicMessage::UpdateSong(song) => self.song = Some(song),
            MusicMessage::UpdateCover(cover_path) => {
                if let Some(song) = &mut self.song {
                    song.cover = Some(load_img(&cover_path))
                }
            }
            MusicMessage::UpdateColor(color) => {
                if let Some(song) = &mut self.song {
                    song.color = Some(color)
                }
            }
        }
    }

    pub fn spotify_subscription(&self) -> Subscription<MusicMessage> {
        Subscription::run(init_spotify)
    }
}

fn load_img(path: &Path) -> Handle {
    let mut file = fs::File::open(path).expect("could not open image for loading");
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect("could not read image for loading");
    Handle::from_bytes(contents)
}
