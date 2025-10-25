use color_eyre::Result;
use crossbeam_channel::{Receiver, Sender};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::WidgetRef;
use rodio::Source;
use rusqlite::Connection;
use std::time::Duration;

use crate::components::ComponentCommand;
use crate::components::{
    Component, PlayerControlsComponent, PlaylistComponent, TracklistComponent,
};
use crate::config::Config;
use crate::current_track::CurrentTrack;
use crate::event::{AudioMessage, Command as AudioCommand, EventState, Key};
use crate::io::{add_metadata, get_files};

pub enum Focus {
    Tracklist,
    Playlist,
}

pub struct App {
    tracklist: TracklistComponent,
    playlist: PlaylistComponent,
    player_controls: PlayerControlsComponent,

    current_track: Option<CurrentTrack>,

    focus: Focus,
    sqlite: Connection,

    audio_tx: Sender<AudioCommand>,
    widget_cmd_rx: Receiver<ComponentCommand>,

    pub config: Config,
}

impl App {
    pub fn new(audio_tx: Sender<AudioCommand>, config: Config, sqlite: Connection) -> Result<Self> {
        let paths = get_files(&config.audio_dir, "mp3")?;
        let tracks = add_metadata(paths);

        let (app_cmd_tx, app_cmd_rx) = crossbeam_channel::bounded(256);

        Ok(App {
            tracklist: TracklistComponent::new(
                tracks,
                config.key_config.clone(),
                app_cmd_tx.clone(),
            ),
            playlist: PlaylistComponent {},
            player_controls: PlayerControlsComponent::new(),
            current_track: None,
            focus: Focus::Tracklist,
            sqlite,
            audio_tx,
            widget_cmd_rx: app_cmd_rx,
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
        let res = self.component_event(key);
        self.drain_commands()?;
        res
    }

    pub fn tick(&mut self) -> Result<()> {
        self.audio_tx.send(AudioCommand::SendState)?;
        Ok(())
    }

    pub fn audio(&mut self, audio_message: AudioMessage) {
        match audio_message {
            AudioMessage::EndOfTrack => {
                self.player_controls.progress = 0;
                self.player_controls.name = None;
                self.current_track = None;
            }
            AudioMessage::State(state) => {
                let progress = if let Some(current_track) = self.current_track.as_ref() {
                    (state.pos.as_secs_f32() / current_track.total_duration.as_secs_f32() * 100.0)
                        .ceil() as u16
                } else {
                    0
                };

                self.player_controls.progress = progress;
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

    fn drain_commands(&mut self) -> Result<()> {
        while let Ok(cmd) = self.widget_cmd_rx.try_recv() {
            match cmd {
                ComponentCommand::TracklistComponent(cmd) => {
                    use crate::components::tracklist::Command;
                    match cmd {
                        Command::SetCurrentTrack { path } => {
                            let file = std::fs::File::open(&path)?;
                            let source = rodio::Decoder::new(file)?;

                            self.current_track = Some(CurrentTrack {
                                path,
                                total_duration: source.total_duration().unwrap_or(Duration::ZERO),
                            });

                            self.player_controls.name =
                                Some(self.current_track.as_ref().unwrap().name());

                            _ = self.audio_tx.send(AudioCommand::Play(Box::new(source)));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
