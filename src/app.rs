use crate::{
    components::Component,
    event::{EventState, Key},
};
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::WidgetRef,
};

use rusqlite::Connection;

use crate::{
    components::{PlayerControlsComponent, PlaylistComponent, TracklistComponent},
    config::Config,
    event::Command,
    io::{add_metadata, get_files},
};

pub enum Focus {
    Tracklist,
    Playlist,
}

pub struct App {
    tracklist: TracklistComponent,
    playlist: PlaylistComponent,
    player_controls: PlayerControlsComponent,

    focus: Focus,
    sqlite: Connection,

    audio_tx: crossbeam_channel::Sender<Command>,

    pub config: Config,
}

impl App {
    pub fn new(
        audio_tx: crossbeam_channel::Sender<Command>,
        config: Config,
        sqlite: Connection,
    ) -> Result<Self> {
        let paths = get_files(&config.audio_dir, "mp3")?;
        let tracks = add_metadata(paths);

        Ok(App {
            tracklist: TracklistComponent::new(tracks, config.key_config.clone(), audio_tx.clone()),
            playlist: PlaylistComponent {},
            player_controls: PlayerControlsComponent {},
            focus: Focus::Tracklist,
            sqlite,
            audio_tx,
            config,
        })
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        match self.focus {
            Focus::Tracklist => self.tracklist.render_ref(area, buf),
            Focus::Playlist => unimplemented!(),
        }
    }

    pub fn event(&mut self, key: Key) -> Result<EventState> {
        self.component_event(key)
    }

    fn component_event(&mut self, key: Key) -> Result<EventState> {
        match self.focus {
            Focus::Tracklist => self.tracklist.event(key),
            Focus::Playlist => unimplemented!(),
        }
    }
}
