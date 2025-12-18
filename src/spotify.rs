use iced::{
    Color,
    futures::{SinkExt, Stream, StreamExt, join},
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
use std::{pin::pin, sync::Arc};

use crate::http_cache;
use crate::music::{MusicMessage, Song};

const CACHE: &str = ".cache";
const CACHE_FILES: &str = ".cache/files";
// 100MB
const CACHE_SIZE: u64 = 1024 * 1024 * 100;

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

pub fn init_spotify() -> impl Stream<Item = MusicMessage> {
    let session_config = SessionConfig::default();
    let player_config = PlayerConfig::default();
    let audio_format = AudioFormat::default();
    let connect_config = ConnectConfig {
        disable_volume: false,
        name: "Ethist Display".to_string(),
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
