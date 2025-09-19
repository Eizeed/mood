use crate::{components::Widget, event::{EventState, Key}};
use color_eyre::Result;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Paragraph};
use std::{path::PathBuf, rc::Rc};

use rusqlite::Connection;

use crate::{
    components::{PlayerControlsComponent, PlaylistComponent, TracklistComponent},
    config::Config,
    event::Command,
    io::{add_metadata, get_files},
    models::Track,
};

pub enum Focus {
    Tracklist,
    Playlist,
}

pub struct App {
    library: Rc<Vec<Track>>,

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
            library: Rc::new(tracks),
            tracklist: TracklistComponent {},
            playlist: PlaylistComponent {},
            player_controls: PlayerControlsComponent {},
            focus: Focus::Tracklist,
            sqlite,
            audio_tx,
            config,
        })
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let tracks = self
            .library
            .iter()
            .take(area.height as usize)
            .map(|t| t.path.to_path_buf().to_string_lossy().to_string())
            .collect::<Vec<String>>();

        Paragraph::new(tracks.join("\n")).render(area, buf);
    }

    pub fn event(&mut self, event: Key) -> Result<EventState> {
        Ok(EventState::NotConsumed)
    }
}
