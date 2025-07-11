use std::{
    collections::VecDeque,
    io::{BufReader, stdout},
    path::Path,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender};

use ratatui::{
    Terminal,
    crossterm::{
        ExecutableCommand,
        event::{
            DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
            MouseEventKind,
        },
    },
    layout::{Constraint, Direction, Layout},
    prelude::Backend,
    text::{Line, Text},
    widgets::Widget,
};
use rodio::Source;

use crate::{
    input::spawn_input,
    music::{Command, Message, spawn_music},
    widget::{ControlBar, Player, control_bar::Repeat, player::Track},
};

pub struct App {
    player: Player,
    control_bar: ControlBar,
    should_exit: bool,
    volume: f32,

    start_timer: Instant,

    pub audio_tx: Sender<Command>,
    pub audio_rx: Receiver<Message>,

    pub input_rx: Receiver<Event>,
}

impl App {
    pub fn with_player(player: Player) -> Self {
        // Idk how much to allocate
        let (main_audio_tx, main_audio_rx) = crossbeam_channel::bounded::<Command>(64);
        let (audio_main_tx, audio_main_rx) = crossbeam_channel::bounded::<Message>(64);

        let (input_main_tx, input_main_rx) = crossbeam_channel::bounded::<Event>(64);

        spawn_music(main_audio_rx, audio_main_tx);
        spawn_input(input_main_tx);

        let Message::CurrentVolume(volume) = audio_main_rx.recv().unwrap() else {
            unreachable!("How tf you here (Recieved not Message::CurrentVolume)");
        };

        App {
            player,
            volume,
            control_bar: ControlBar::new(),
            start_timer: Instant::now(),
            audio_tx: main_audio_tx,
            audio_rx: audio_main_rx,
            input_rx: input_main_rx,
            should_exit: false,
        }
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn start<B: Backend>(mut self, mut terminal: Terminal<B>) {
        stdout().execute(EnableMouseCapture).unwrap();

        loop {
            terminal
                .draw(|f| self.render(f.area(), f.buffer_mut()))
                .expect("failed to draw frame");

            loop {
                crossbeam_channel::select_biased! {
                    recv(self.audio_rx) -> msg => {
                        let msg = msg.unwrap();

                        self.handle_audio_rx(msg);

                        for msg in self.audio_rx.clone().try_iter() {
                            self.handle_audio_rx(msg)
                        }

                        break;
                    }
                    recv(self.input_rx) -> event => {
                        self.handle_event(event.unwrap());
                        break;
                    }
                };
            }

            if self.should_exit() {
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

                match keycode {
                    KeyCode::Esc => self.should_exit = true,
                    KeyCode::Char('k') | KeyCode::Up => {
                        match modifiers {
                            KeyModifiers::CONTROL => {
                                let vol = 0.05;
                                self.audio_tx.send(Command::VolumeUp(vol)).unwrap();
                            }
                            KeyModifiers::NONE => {
                                self.player
                                    .set_cursor(self.player.cursor().saturating_sub(1));
                            }
                            _ => (),
                        };
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        match modifiers {
                            KeyModifiers::CONTROL => {
                                self.audio_tx.send(Command::VolumeDown(0.05)).unwrap();
                            }
                            KeyModifiers::NONE => {
                                let new_y = self.player.cursor() + 1;
                                if self.player.tracks_len() as u16 > new_y {
                                    self.player.set_cursor(new_y);
                                }
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
                        let track = self.player.track_under_cursor().clone();
                        let index = self.player.cursor();
                        self.player.manual_queue_mut().push_back(Track {
                            path: track,
                            index: index as usize,
                        });
                    }
                    KeyCode::Enter => {
                        let index = self.player.cursor() as usize;

                        self.player.set_index(index);

                        let track = self.set_auto_queue(index);
                        self.set_audio(track.path, track.index);
                    }
                    KeyCode::Char(' ') => self.toggle_pause(),
                    _ => {}
                }
            }
            Event::Mouse(m) => {
                let x = m.column;
                let y = m.row;

                match m.kind {
                    MouseEventKind::Down(button) => match button {
                        MouseButton::Left => {
                            if y == self.control_bar.control_bar_y() {
                                if self.control_bar.shuffle_pos().contains(&x) {
                                    self.control_bar.random = !self.control_bar.random;
                                }

                                if self.control_bar.seek_backward_pos().contains(&x) {
                                    self.seek_backward(Duration::from_secs(5));
                                }

                                if self.control_bar.pause_pos().contains(&x) {
                                    self.toggle_pause();
                                }

                                if self.control_bar.seek_forward_pos().contains(&x) {
                                    self.seek_forward(Duration::from_secs(5));
                                }

                                if self.control_bar.repeat_pos().contains(&x) {
                                    self.control_bar.toggle_repeat();
                                }
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_audio_rx(&mut self, message: Message) {
        match message {
            Message::TrackEnded => {
                if let Repeat::RepeatOne = self.control_bar.repeat {}

                if let Repeat::RepeatQueue = self.control_bar.repeat {
                    match self.player.auto_queue_mut().pop_front() {
                        Some(track) => {
                            self.set_audio(track.path, track.index);

                            return;
                        }
                        None => {}
                    }
                }

                let (path, index) = if self.player.manual_queue_mut().is_empty() {
                    match self.control_bar.repeat {
                        Repeat::None => {
                            let track = match self.player.auto_queue_mut().pop_front() {
                                Some(track) => track,
                                None => {
                                    self.player.unset_current();
                                    self.control_bar.progress = None;
                                    return;
                                }
                            };

                            (track.path, track.index)
                        }
                        Repeat::RepeatQueue => {
                            let track = match self.player.auto_queue_mut().pop_front() {
                                Some(track) => track,
                                None => self.set_auto_queue(0),
                            };

                            (track.path, track.index)
                        }
                        Repeat::RepeatOne => {
                            let path = self.player.get_current_path().unwrap().to_string();
                            let index = self.player.get_current_index().unwrap();
                            (path, index)
                        }
                    }
                } else {
                    let next_track = self.player.manual_queue_mut().pop_front().unwrap();

                    (next_track.path, next_track.index)
                };

                self.set_audio(path, index);
            }
            Message::CurrentVolume(vol) => {
                self.volume = vol;
            }
            Message::CurrentPos(pos) => {
                let dur = self.player.get_current_duration();
                match dur {
                    Some(dur) => {
                        self.control_bar.progress =
                            Some((pos.as_millis() as f32) / (dur.as_millis() as f32));
                    }
                    None => {}
                }
            }
        };
    }

    pub fn set_audio<T: AsRef<Path>>(&mut self, path: T, index: usize) {
        let file = std::fs::File::open(&path).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        let duration = source.total_duration().unwrap_or(Duration::from_secs(0));

        self.audio_tx.send(Command::play(source)).unwrap();
        self.start_timer = Instant::now();

        let name = path
            .as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(path.as_ref().to_str().unwrap());

        self.control_bar.name = name.to_string();

        self.control_bar.progress = Some(0.0);
        self.player
            .set_current(path.as_ref().to_str().unwrap().to_string(), duration, index);
    }

    pub fn set_auto_queue(&mut self, index: usize) -> Track {
        let tracks: Vec<Track> = self
            .player
            .tracks()
            .iter()
            .enumerate()
            .filter(|(idx, _str)| *idx >= index)
            .map(|(idx, str)| Track {
                path: str.clone(),
                index: idx,
            })
            .collect();

        self.player.set_auto_queue(tracks.into_iter());
        self.player.auto_queue_mut().pop_front().unwrap()
    }

    pub fn seek_backward(&mut self, duration: Duration) {
        if self.start_timer.elapsed() < Duration::from_millis(100) {
            return;
        }
        self.audio_tx.send(Command::SeekBackward(duration)).unwrap();
    }

    pub fn seek_forward(&mut self, duration: Duration) {
        if self.start_timer.elapsed() < Duration::from_millis(100) {
            return;
        }
        self.audio_tx.send(Command::SeekForward(duration)).unwrap();
    }

    pub fn toggle_pause(&mut self) {
        if self.player.is_paused() {
            self.audio_tx.send(Command::resume()).unwrap();
            self.player.set_is_paused(false);
        } else {
            self.audio_tx.send(Command::pause()).unwrap();
            self.player.set_is_paused(true);
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let [header_area, main_area, control_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(3),
            ],
        )
        .areas(area);

        let current = Line::raw(self.player.get_current_path().unwrap_or("No track"));
        let vol = Line::raw(format!("Volume: {:.0}%", self.volume * 100.0));

        let header = Text::from_iter(vec![current, vol]);

        header.render(header_area, buf);

        self.player.render(main_area, buf);
        self.control_bar.render(control_area, buf);
    }
}
