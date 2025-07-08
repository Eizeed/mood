use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};

use ratatui::{
    Terminal,
    crossterm::event::{Event, KeyCode, KeyModifiers},
    layout::{Constraint, Direction, Layout},
    prelude::Backend,
    text::{Line, Text},
    widgets::Widget,
};

use crate::{
    input::spawn_input,
    music::{Command, Message, spawn_music},
    widget::Player,
};

pub struct App {
    player: Player,
    should_exit: bool,
    volume: f32,

    pub audio_tx: Sender<Command>,
    pub audio_rx: Receiver<Message>,

    pub input_rx: Receiver<Event>,
}

impl App {
    pub fn with_player(player: Player) -> Self {
        let (main_audio_tx, main_audio_rx) = crossbeam_channel::bounded::<Command>(1024);
        let (audio_main_tx, audio_main_rx) = crossbeam_channel::bounded::<Message>(1024);

        let (input_main_tx, input_main_rx) = crossbeam_channel::bounded::<Event>(1024);

        spawn_music(main_audio_rx, audio_main_tx);
        spawn_input(input_main_tx);

        let Message::CurrentVolume(volume) = audio_main_rx.recv().unwrap() else {
            unreachable!()
        };

        App {
            player,
            volume,
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
                        let msg = msg.unwrap();
                        match msg {
                            Message::TrackEnded => {
                                self.player.unset_current();
                                break;
                            },
                            Message::CurrentVolume(vol) => {
                                self.volume = vol;
                                break;
                            }
                        };
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
                            self.audio_tx
                                .send(Command::SeekBackward(Duration::from_secs(5)))
                                .unwrap();
                        }
                        _ => {}
                    },
                    KeyCode::Char('l') => match modifiers {
                        KeyModifiers::CONTROL => {
                            self.audio_tx
                                .send(Command::SeekForward(Duration::from_secs(5)))
                                .unwrap();
                        }
                        _ => {}
                    },
                    KeyCode::Enter => {
                        self.player.set_current(self.player.track_under_cursor());

                        self.audio_tx
                            .send(Command::play(self.player.get_current().unwrap()))
                            .unwrap();
                    }
                    KeyCode::Char(' ') => {
                        if self.player.is_paused() {
                            self.audio_tx.send(Command::pause()).unwrap();
                            self.player.set_is_paused(false);
                        } else {
                            self.audio_tx.send(Command::resume()).unwrap();
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
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let [header_area, main_area] = Layout::new(
            Direction::Vertical,
            [Constraint::Length(2), Constraint::Length(4)],
        )
        .areas(area);

        let current = Line::raw(self.player.get_current().unwrap_or("No track"));
        let vol = Line::raw(format!("Volume: {:.0}%", self.volume * 100.0));

        let header = Text::from_iter(vec![current, vol]);

        header.render(header_area, buf);

        self.player.render(main_area, buf);
    }
}
