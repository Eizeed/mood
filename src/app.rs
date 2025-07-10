use std::{
    io::BufReader,
    ops::ControlFlow,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, RecvError, Sender};

use ratatui::{
    Terminal,
    crossterm::event::{Event, KeyCode, KeyModifiers},
    layout::{Constraint, Direction, Layout},
    prelude::Backend,
    text::{Line, Text},
    widgets::Widget,
};
use rodio::Source;

use crate::{
    input::spawn_input,
    music::{Command, Message, spawn_music},
    widget::{Player, player::Track},
};

pub struct App {
    player: Player,
    should_exit: bool,
    volume: f32,

    progress: Option<f32>,

    start_timer: Instant,

    pub audio_tx: Sender<Command>,
    pub audio_rx: Receiver<Message>,

    pub input_rx: Receiver<Event>,
}

impl App {
    pub fn with_player(player: Player) -> Self {
        let (main_audio_tx, main_audio_rx) = crossbeam_channel::bounded::<Command>(64);
        let (audio_main_tx, audio_main_rx) = crossbeam_channel::bounded::<Message>(64);

        let (input_main_tx, input_main_rx) = crossbeam_channel::bounded::<Event>(64);

        spawn_music(main_audio_rx, audio_main_tx);
        spawn_input(input_main_tx);

        let Message::CurrentVolume(volume) = audio_main_rx.recv().unwrap() else {
            unreachable!()
        };

        App {
            player,
            volume,
            progress: None,
            start_timer: Instant::now(),
            audio_tx: main_audio_tx,
            audio_rx: audio_main_rx,
            input_rx: input_main_rx,
            should_exit: false,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn start<B: Backend>(mut self, mut terminal: Terminal<B>) {
        loop {
            terminal
                .draw(|f| {
                    self.render(f.area(), f.buffer_mut());
                })
                .expect("failed to draw frame");

            loop {
                crossbeam_channel::select_biased! {
                    recv(self.audio_rx) -> msg => {
                        match self.handle_audio_rx(msg) {
                            ControlFlow::Break(()) => break,
                            _ => {}
                        }
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
    }

    pub fn handle_event(&mut self, ev: Event) {
        match ev {
            Event::Key(k) => {
                let keycode = k.code;
                let modifiers = k.modifiers;

                match keycode {
                    KeyCode::Esc => self.should_exit = true,
                    KeyCode::Char('k') => {
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
                    KeyCode::Char('j') => {
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
                            if self.start_timer.elapsed() < Duration::from_millis(100) {
                                return;
                            }
                            self.audio_tx
                                .send(Command::SeekBackward(Duration::from_secs(5)))
                                .unwrap();
                        }
                        _ => {}
                    },
                    KeyCode::Char('l') => match modifiers {
                        KeyModifiers::CONTROL => {
                            if self.start_timer.elapsed() < Duration::from_millis(100) {
                                return;
                            }
                            self.audio_tx
                                .send(Command::SeekForward(Duration::from_secs(5)))
                                .unwrap();
                        }
                        _ => {}
                    },
                    KeyCode::Char('q') => {
                        let track = self.player.track_under_cursor().clone();
                        let index = self.player.cursor();
                        self.player.queue_mut().push_back(Track {
                            path: track,
                            index: index as usize,
                        });
                    }
                    KeyCode::Enter => {
                        let path = self.player.track_under_cursor();
                        let file = std::fs::File::open(path).unwrap();
                        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

                        self.player.set_current(
                            self.player.track_under_cursor(),
                            source.total_duration().unwrap_or(Duration::from_secs(0)),
                            self.player.cursor(),
                        );

                        self.player.set_index(self.player.cursor() as usize);

                        self.audio_tx.send(Command::play(source)).unwrap();
                        self.start_timer = Instant::now();
                    }
                    KeyCode::Char(' ') => {
                        if self.player.is_paused() {
                            self.audio_tx.send(Command::resume()).unwrap();
                            self.player.set_is_paused(false);
                        } else {
                            self.audio_tx.send(Command::pause()).unwrap();
                            self.player.set_is_paused(true);
                        }
                    }
                    _ => {}
                }
            }
            Event::Mouse(_m) => {}
            _ => {}
        }
    }

    fn handle_audio_rx(&mut self, msg: Result<Message, RecvError>) -> ControlFlow<()> {
        let msg = msg.unwrap();

        let mut handle = |msg: Message| {
            match msg {
                Message::TrackEnded => {
                    if self.player.queue_mut().is_empty() {
                        if self.player.index().unwrap() >= self.player.tracks_len() - 1 {
                            self.player.set_index(0);
                        } else {
                            self.player.set_index(self.player.index().unwrap() + 1);
                        };

                        let index = self.player.index().unwrap();

                        let path = self.player.tracks()[index].clone();
                        let file = std::fs::File::open(&path).unwrap();
                        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                        let duration = source.total_duration().unwrap_or(Duration::from_secs(0));

                        self.audio_tx.send(Command::play(source)).unwrap();
                        self.start_timer = Instant::now();

                        self.progress = Some(0.0);
                        self.player.set_current(path, duration, index);
                    } else {
                        let next_track = self.player.queue_mut().pop_front().unwrap();
                        let file = std::fs::File::open(&next_track.path).unwrap();
                        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                        let duration = source.total_duration().unwrap_or(Duration::from_secs(0));

                        self.audio_tx.send(Command::play(source)).unwrap();

                        self.progress = Some(0.0);
                        self.player
                            .set_current(next_track.path, duration, next_track.index);
                    }
                }
                Message::CurrentVolume(vol) => {
                    self.volume = vol;
                }
                Message::CurrentPos(pos) => {
                    let dur = self.player.get_current_duration();
                    match dur {
                        Some(dur) => {
                            self.progress =
                                Some((pos.as_millis() as f32) / (dur.as_millis() as f32));
                        }
                        None => {}
                    }
                }
            };
        };

        handle(msg);

        for msg in self.audio_rx.try_iter() {
            handle(msg)
        }

        ControlFlow::Break(())
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
                Constraint::Length(2),
            ],
        )
        .areas(area);

        let current = Line::raw(self.player.get_current_path().unwrap_or("No track"));
        let vol = Line::raw(format!("Volume: {:.0}%", self.volume * 100.0));

        let header = Text::from_iter(vec![current, vol]);

        let [buttons_area, progress_area] = Layout::new(
            Direction::Vertical,
            [Constraint::Length(1), Constraint::Length(1)],
        )
        .areas(control_area);

        let [_, progress_area, _] = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Fill(1),
                Constraint::Fill(8),
                Constraint::Fill(1),
            ],
        )
        .areas(progress_area);

        {
            let width = progress_area.width;
            let progress = match self.progress {
                Some(progress) => progress * 100.0,
                None => 0.0,
            };

            let one_cell_rat = 100.0 / width as f32;

            let till = (progress / one_cell_rat).round() as u16;
            for x in 0..width {
                let cell = buf
                    .cell_mut((progress_area.x + x, progress_area.y))
                    .unwrap();
                if x < till {
                    cell.set_char('#');
                } else {
                    cell.set_char('_');
                }
            }
        };

        Line::raw(format!("{:?}", self.progress)).render(buttons_area, buf);

        header.render(header_area, buf);

        self.player.render(main_area, buf);
    }
}
