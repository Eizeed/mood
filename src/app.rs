use color_eyre::Result;
use std::rc::Rc;

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
}
