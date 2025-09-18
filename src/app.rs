use rusqlite::Connection;

use crate::{
    components::{PlayerControlsComponent, PlaylistComponent, TracklistComponent},
    config::Config,
    event::Command,
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
    ) -> Self {
        App {
            tracklist: TracklistComponent {},
            playlist: PlaylistComponent {},
            player_controls: PlayerControlsComponent {},
            focus: Focus::Tracklist,
            sqlite,
            audio_tx,
            config,
        }
    }
}
