use crate::{
    components::Component,
    event::{AudioMessage, EventState, Key},
};
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
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
            player_controls: PlayerControlsComponent::new(),
            focus: Focus::Tracklist,
            sqlite,
            audio_tx,
            config,
        })
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let [main_area, controls_area] = Layout::new(
            Direction::Vertical,
            [Constraint::Fill(1), Constraint::Length(5)],
        )
        .areas(area);

        self.player_controls.render_ref(controls_area, buf);

        match self.focus {
            Focus::Tracklist => self.tracklist.render_ref(main_area, buf),
            Focus::Playlist => unimplemented!(),
        }
    }

    pub fn event(&mut self, key: Key) -> Result<EventState> {
        self.component_event(key)
    }

    pub fn tick(&mut self) {
        _ = self.audio_tx.send(Command::SendState);
    }

    pub fn audio(&mut self, audio_message: AudioMessage) {
        match audio_message {
            AudioMessage::EndOfTrack => {}
            AudioMessage::State(state) => {
                let progress = if let Some(d) = state.total_duraiton {
                    (state.pos.as_secs_f32() / d.as_secs_f32() * 100.0).ceil() as u16
                } else {
                    0
                };

                self.player_controls.progress.set(progress);
            }
            AudioMessage::Noop => {}
        }
    }

    fn component_event(&mut self, key: Key) -> Result<EventState> {
        match self.focus {
            Focus::Tracklist => self.tracklist.event(key),
            Focus::Playlist => unimplemented!(),
        }
    }
}
