use std::{
    io::{BufReader, stdout},
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender};

use rand::seq::SliceRandom;
use ratatui::{
    Terminal,
    crossterm::{
        ExecutableCommand,
        event::{
            DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
            MouseButton, MouseEventKind,
        },
    },
    layout::Rect,
    prelude::Backend,
    widgets::{StatefulWidget, Widget},
};
use rusqlite::Connection;
use uuid::Uuid;

use crate::{
    config::Config,
    input::spawn_input,
    io::{add_uuid_metadata, get_files},
    model::{self, Track, playlist::DbTrack},
    music::{Command, Message, spawn_music},
    screen::player::{Focus, Player},
};

pub struct App {
    paused: bool,
    progress: f32,
    volume: f32,
    repeat: Repeat,
    shuffle: Shuffle,

    should_exit: bool,

    player: Player,

    tracks: Vec<Track>,

    start_timer: Instant,
    last_seek_timer: Instant,

    audio_tx: Sender<Command>,
    audio_rx: Receiver<Message>,

    input_rx: Receiver<Event>,

    db_conn: Connection,
}

#[derive(Debug, Clone, Copy)]
pub enum Repeat {
    None,
    Queue,
    One,
}

#[derive(Debug, Clone, Copy)]
pub enum Shuffle {
    None,
    Random,
}

impl App {
    pub fn new(config: Config, area: Rect) -> Self {
        let conn = Connection::open(config.database_path).unwrap();

        let (main_audio_tx, main_audio_rx) = crossbeam_channel::bounded::<Command>(64);
        let (audio_main_tx, audio_main_rx) = crossbeam_channel::bounded::<Message>(64);

        let (input_main_tx, input_main_rx) = crossbeam_channel::bounded::<Event>(64);

        spawn_music(main_audio_rx, audio_main_tx);
        spawn_input(input_main_tx);

        main_audio_tx
            .send(Command::SetVolume(config.volume))
            .unwrap();

        let paths = get_files(config.audio_dir_path, "mp3");

        let tracks = add_uuid_metadata(paths);

        conn.execute(
            r#"
                CREATE TABLE IF NOT EXISTS tracks (
                    uuid        TEXT PRIMARY KEY,
                    path        TEXT NOT NULL UNIQUE
                );
            "#,
            (),
        )
        .unwrap();

        conn.execute(
            r#"
                CREATE TABLE IF NOT EXISTS playlist (
                    uuid        TEXT PRIMARY KEY,
                    name        TEXT NOT NULL UNIQUE
                );
            "#,
            (),
        )
        .unwrap();

        conn.execute(
            r#"
                CREATE TABLE IF NOT EXISTS playlist_tracks (
                    playlist_uuid        TEXT NOT NULL,
                    track_uuid           TEXT NOT NULL,

                    PRIMARY KEY (playlist_uuid, track_uuid)
                );
            "#,
            (),
        )
        .unwrap();

        let uuids: Vec<Uuid> = {
            let mut stmt = conn.prepare("SELECT uuid, path FROM tracks;").unwrap();

            stmt.query_map((), |row| row.get("uuid"))
                .unwrap()
                .map(|r: Result<Box<str>, _>| Uuid::parse_str(&r.unwrap()).unwrap())
                .collect()
        };

        tracks
            .iter()
            .filter(|track| uuids.iter().find(|uuid| **uuid == track.uuid).is_none())
            .for_each(|t| {
                eprintln!("Added track to database: {:?}", t.path.file_name().unwrap());
                conn.execute(
                    "INSERT INTO playlists (uuid, path) VALUES (?1, ?2)",
                    (t.uuid.to_string(), t.path.to_string_lossy()),
                )
                .unwrap();
            });

        let playlists: Vec<model::Playlist> = {
            let mut stmt = conn.prepare("SELECT uuid, name FROM playlists;").unwrap();

            stmt.query_map((), |row| {
                let uuid: Box<str> = row.get("uuid")?;
                let uuid = Uuid::parse_str(&uuid).unwrap();

                let name: String = row.get("name")?;

                Ok(model::Playlist { uuid, name })
            })
            .unwrap()
            .into_iter()
            .map(|r| r.unwrap())
            .collect()
        };

        let player = Player::new(tracks.clone(), playlists, area);

        App {
            paused: false,
            progress: 0.0,
            volume: config.volume,
            repeat: config.repeat,
            shuffle: config.shuffle,

            should_exit: false,

            tracks,

            player,
            start_timer: Instant::now(),
            last_seek_timer: Instant::now(),

            audio_tx: main_audio_tx,
            audio_rx: audio_main_rx,
            input_rx: input_main_rx,

            db_conn: conn,
        }
    }

    pub fn context(&self) -> Context {
        Context {
            paused: self.paused,
            volume: self.volume,
            repeat: self.repeat,
            shuffle: self.shuffle,
            progress: self.progress,
        }
    }

    pub fn start<B: Backend>(mut self, mut terminal: Terminal<B>) {
        stdout().execute(EnableMouseCapture).unwrap();

        loop {
            terminal
                .draw(|f| self.render(f.area(), f.buffer_mut()))
                .expect("failed to draw frame");

            loop {
                crossbeam_channel::select_biased! {
                    recv(self.input_rx) -> event => {
                        self.handle_event(event.unwrap());
                        break;
                    }
                    recv(self.audio_rx) -> msg => {
                        let msg = msg.unwrap();

                        self.handle_audio_rx(msg);

                        for msg in self.audio_rx.clone().try_iter() {
                            self.handle_audio_rx(msg)
                        }

                        break;
                    }
                };
            }

            if self.should_exit {
                break;
            }
        }
        stdout().execute(DisableMouseCapture).unwrap();
    }

    pub fn handle_event(&mut self, ev: Event) {
        match ev {
            Event::Key(k) => {
                let keycode = k.code;
                let modifiers = k.modifiers;

                if KeyEventKind::Release == k.kind {
                    return;
                };

                match keycode {
                    KeyCode::Esc => self.should_exit = true,
                    KeyCode::Char('k') | KeyCode::Up => {
                        match modifiers {
                            KeyModifiers::CONTROL => {
                                let vol = 0.05;
                                self.audio_tx.send(Command::volume_up(vol)).unwrap();
                            }
                            KeyModifiers::NONE => {
                                self.player.cursor_up(1);
                            }
                            _ => (),
                        };
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        match modifiers {
                            KeyModifiers::CONTROL => {
                                self.audio_tx.send(Command::volume_down(0.05)).unwrap();
                            }
                            KeyModifiers::NONE => {
                                self.player.cursor_down(1);
                            }
                            _ => (),
                        };
                    }
                    KeyCode::Char('h') => match modifiers {
                        KeyModifiers::CONTROL => {
                            self.seek_backward(Duration::from_secs(5));
                        }
                        _ => {}
                    },
                    KeyCode::Char('l') => match modifiers {
                        KeyModifiers::CONTROL => {
                            self.seek_forward(Duration::from_secs(5));
                        }
                        _ => {}
                    },
                    KeyCode::Char('q') => {
                        let track = self.player.get_under_cursor();
                        self.player.push_front_manual_queue(track);
                    }
                    KeyCode::Enter => {
                        match self.player.focused_widget {
                            Focus::Tracklist => {
                                let track = self.player.get_under_cursor();

                                match self.shuffle {
                                    Shuffle::Random => {
                                        self.player.tracklist.list =
                                            self.player.tracklist.base.clone();

                                        self.player.tracklist.list.swap(track.index, 0);
                                        self.player.tracklist.list[1..].shuffle(&mut rand::rng());
                                        self.player.set_auto_queue(0);
                                    }
                                    _ => {
                                        let index = (self.player.tracklist.cursor
                                            + self.player.tracklist.y_offset)
                                            as usize;

                                        self.player.set_auto_queue(index);
                                    }
                                };

                                // TODO: Handle option
                                let track = self.player.pop_auto_queue().unwrap();

                                self.play(track);
                            }
                            Focus::Playlist => {
                                let playlist = self.player.playlist.get_under_cursor();
                                let db_tracks =
                                    DbTrack::get_by_playlist_uuid(&self.db_conn, playlist.uuid);

                                let tracks =
                                    Track::from_db_tracks(&self.player.tracklist.base, db_tracks);

                                self.player.tracklist.base = tracks.clone();
                                self.player.tracklist.list = tracks;
                                self.player.tracklist.history.clear();
                                self.player.tracklist.auto_queue.clear();

                                self.player.switch_window();
                            }
                        }
                    }
                    KeyCode::Char(' ') => self.toggle_pause(),
                    KeyCode::Char('p') => {
                        self.player.switch_window();
                    }
                    _ => {}
                }
            }
            Event::Mouse(m) => {
                let x = m.column;
                let y = m.row;

                match m.kind {
                    MouseEventKind::Down(button) => match button {
                        MouseButton::Left => {
                            if y == self.player.control_bar.control_bar_y {
                                if self.player.control_bar.shuffle_pos.contains(&x) {
                                    match self.shuffle {
                                        Shuffle::None => self.shuffle = Shuffle::Random,
                                        Shuffle::Random => self.shuffle = Shuffle::None,
                                    }
                                }

                                if self.player.control_bar.seek_backward_pos.contains(&x) {
                                    self.play_prev();
                                }

                                if self.player.control_bar.pause_pos.contains(&x) {
                                    self.toggle_pause();
                                }

                                if self.player.control_bar.seek_forward_pos.contains(&x) {
                                    self.play_next();
                                }

                                if self.player.control_bar.repeat_pos.contains(&x) {
                                    match self.repeat {
                                        Repeat::None => self.repeat = Repeat::Queue,
                                        Repeat::Queue => self.repeat = Repeat::One,
                                        Repeat::One => self.repeat = Repeat::None,
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            Event::Resize(c, r) => {
                self.player.resize(c, r);
            }
            _ => {}
        };
    }

    fn handle_audio_rx(&mut self, message: Message) {
        match message {
            Message::TrackEnded => {
                if let Repeat::One = self.repeat {
                    let current = self.player.get_current().unwrap().clone();
                    self.play(current);

                    return;
                };

                let Some(track) = self.player.get_next(self.repeat) else {
                    return;
                };

                self.play(track);
            }
            Message::CurrentVolume(vol) => {
                self.volume = vol;
            }
            Message::CurrentPos(pos) => {
                let dur = self.player.get_current().map(|c| c.duration);
                match dur {
                    Some(dur) => {
                        self.progress = (pos.as_millis() as f32) / (dur.as_millis() as f32);
                    }
                    None => {}
                }
            }
        };
    }

    pub fn play(&mut self, track: Track) {
        let file = std::fs::File::open(&track.path).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

        self.audio_tx.send(Command::play(source)).unwrap();
        self.start_timer = Instant::now();

        self.player.set_current(track);
    }

    pub fn seek_backward(&mut self, duration: Duration) {
        if self.start_timer.elapsed() < Duration::from_millis(100) {
            return;
        }
        if self.last_seek_timer.elapsed() < Duration::from_millis(30) {
            return;
        }
        self.audio_tx
            .send(Command::seek_backward(duration))
            .unwrap();
        self.last_seek_timer = Instant::now()
    }

    pub fn seek_forward(&mut self, duration: Duration) {
        if self.start_timer.elapsed() < Duration::from_millis(100) {
            return;
        }
        if self.last_seek_timer.elapsed() < Duration::from_millis(30) {
            return;
        }
        self.audio_tx.send(Command::seek_forward(duration)).unwrap();
        self.last_seek_timer = Instant::now()
    }

    pub fn play_next(&mut self) {
        let track = match self.player.get_next(self.repeat) {
            Some(track) => track,
            None => {
                self.player.set_auto_queue(0);
                let Some(track) = self.player.get_next(self.repeat) else {
                    return;
                };

                track
            }
        };

        self.play(track);
    }

    pub fn play_prev(&mut self) {
        let track = match self.player.get_prev() {
            Some(track) => track,
            None => {
                self.player.tracklist.history = self.player.tracklist.list.clone();

                let Some(track) = self.player.get_prev() else {
                    return;
                };

                track
            }
        };

        self.play(track);
    }

    pub fn toggle_pause(&mut self) {
        if self.paused {
            self.audio_tx.send(Command::resume()).unwrap();
            self.paused = false;
        } else {
            self.audio_tx.send(Command::pause()).unwrap();
            self.paused = true;
        }
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut context = self.context();
        self.player.render(area, buf, &mut context);
    }
}

pub struct Context {
    pub paused: bool,
    pub volume: f32,
    pub progress: f32,
    pub repeat: Repeat,
    pub shuffle: Shuffle,
}
