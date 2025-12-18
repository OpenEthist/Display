use iced::{
    Alignment, Background, Color, Element, Fill, Subscription,
    futures::{SinkExt, Stream, StreamExt, join},
    widget::{
        Container, Text, column, container,
        image::{Handle, Image},
        text,
    },
};
use librespot::{
    connect::{ConnectConfig, Spirc},
    core::{SpotifyId, cache::Cache, config::SessionConfig, session::Session},
    discovery::Discovery,
    metadata::{Lyrics, audio::UniqueFields},
    playback::mixer::MixerConfig,
    playback::{
        audio_backend,
        config::{AudioFormat, PlayerConfig},
        mixer,
        player::{Player, PlayerEvent},
    },
};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    pin::pin,
    sync::Arc,
};

use crate::http_cache;
use crate::theme::DEFAULT_FONT;

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
    paused: bool,
    name: String,
    artist: String,
    length: u32,
    at: u32,
    cover: Option<Handle>,
    cover_url: String,
    color: Option<Color>,
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
                Text::new(&song.name).font(DEFAULT_FONT).size(28.0),
                Text::new(&song.artist).font(DEFAULT_FONT).size(18.0)
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
        Subscription::run(spotify)
    }
}

fn load_img(path: &Path) -> Handle {
    let mut file = fs::File::open(path).expect("could not open image for loading");
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect("could not read image for loading");
    Handle::from_bytes(contents)
}

fn spotify_init_eventhandler(
    player: Arc<Player>,
    session: &Session,
) -> impl Stream<Item = MusicMessage> {
    iced::stream::channel(100, async |mut output| {
        let player = player;

        while let Some(event) = player.get_player_event_channel().recv().await {
            match event {
                PlayerEvent::TrackChanged { audio_item } => {
                    // Get the artist
                    let artist = match audio_item.unique_fields {
                        UniqueFields::Episode { show_name, .. } => show_name,
                        UniqueFields::Local { artists, .. } => match artists {
                            Some(artists) => artists,
                            None => "local".into(),
                        },
                        UniqueFields::Track { artists, .. } => {
                            let mut actual_artists = Vec::new();
                            for artist in artists.0 {
                                actual_artists.push(artist.name);
                            }

                            actual_artists.join(", ")
                        }
                    };

                    // Get the cover (url)
                    let cover_url = audio_item.covers[0].url.clone();

                    let _ = output
                        .send(MusicMessage::UpdateSong(Song {
                            paused: false,
                            name: audio_item.name,
                            artist,
                            length: audio_item.duration_ms,
                            at: 0,
                            cover: None,
                            cover_url: cover_url.clone(),
                            color: None,
                        }))
                        .await;

                    let url = audio_item.track_id.to_id().unwrap();
                    let id: &SpotifyId = &SpotifyId::from_base62(&url)
                        .expect("could not create spotify id from uri");

                    // Get lyrics (and their colors)
                    let mut lyrics_output = output.clone();
                    let mut lyrics_grabber = async || {
                        if let Ok(lyrics) = Lyrics::get(session, id).await {
                            let bg_color = decode_color(lyrics.colors.background);
                            let bg_color =
                                Color::from_rgb8(bg_color.0 / 3, bg_color.1 / 3, bg_color.2 / 3);
                            let _ = lyrics_output
                                .send(MusicMessage::UpdateColor(bg_color))
                                .await;
                        };
                    };

                    // Download the cover
                    let mut cover_output = output.clone();
                    let mut cover_grabber = async || {
                        if let Ok(cover_path) = http_cache::get(&cover_url).await {
                            let _ = cover_output
                                .send(MusicMessage::UpdateCover(cover_path))
                                .await;
                        };
                    };

                    join![lyrics_grabber(), cover_grabber()];
                }
                PlayerEvent::Stopped { .. } => {
                    let _ = output.send(MusicMessage::RemoveSong).await;
                }
                _ => {}
            };
        }
    })
}

// AI Generated
fn decode_color(color: i32) -> (u8, u8, u8) {
    let c = color as u32;

    let r = ((c >> 16) & 0xFF) as u8;
    let g = ((c >> 8) & 0xFF) as u8;
    let b = (c & 0xFF) as u8;

    (r, g, b)
}

fn spotify() -> impl Stream<Item = MusicMessage> {
    let session_config = SessionConfig::default();
    let player_config = PlayerConfig::default();
    let audio_format = AudioFormat::default();
    let connect_config = ConnectConfig {
        disable_volume: false,
        name: "Ethist".to_string(),
        ..Default::default()
    };
    let mixer_config = MixerConfig::default();

    let sink_builder = audio_backend::find(Some("pulseaudio".to_string())).unwrap();
    let mixer_builder = mixer::find(None).unwrap();

    let mut discovery = Discovery::builder(
        session_config.device_id.clone(),
        session_config.client_id.clone(),
    )
    .name("Ethist")
    .launch()
    .expect("could not launch mdns spotify discover");

    let cache = Cache::new(
        Some(CACHE),
        Some(CACHE),
        Some(CACHE_FILES),
        Some(CACHE_SIZE),
    )
    .expect("could not get cache");

    iced::stream::channel(100, async move |mut output| {
        while let Some(credentials) = discovery.next().await {
            let mixer = mixer_builder(mixer_config.to_owned()).expect("could not create mixer");
            let session = Session::new(session_config.to_owned(), Some(cache.to_owned()));

            let player = Player::new(
                player_config.to_owned(),
                session.clone(),
                mixer.get_soft_volume(),
                move || sink_builder(None, audio_format),
            );

            let (_spirc, spirc_task) = Spirc::new(
                connect_config.to_owned(),
                session.clone(),
                credentials,
                player.clone(),
                mixer,
            )
            .await
            .expect("could not create spirc");

            let mut event_handler = pin!(spotify_init_eventhandler(player.clone(), &session));

            let handle = async {
                while let Some(event) = event_handler.next().await {
                    output
                        .send(event)
                        .await
                        .expect("send error in spotify stream");
                }
            };

            join!(spirc_task, handle);
        }
    })
}
