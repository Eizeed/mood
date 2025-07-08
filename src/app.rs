use crossbeam_channel::{Receiver, Sender};

use crossterm::event::{Event, KeyCode};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::Backend,
    text::Line,
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

    pub audio_tx: Sender<Command>,
    pub audio_rx: Receiver<Message>,

    pub input_rx: Receiver<Event>,
}

impl App {
    pub fn with_player(player: Player) -> Self {
        let (main_music_tx, main_music_rx) = crossbeam_channel::bounded::<Command>(1024);
        let (music_main_tx, music_main_rx) = crossbeam_channel::bounded::<Message>(1024);

        let (input_main_tx, input_main_rx) = crossbeam_channel::bounded::<Event>(1024);

        spawn_music(main_music_rx, music_main_tx);
        spawn_input(input_main_tx);

        App {
            player,
            audio_tx: main_music_tx,
            audio_rx: music_main_rx,
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

            crossbeam_channel::select_biased! {
                recv(self.audio_rx) -> msg => {
                    let msg = msg.unwrap();
                    match msg {
                        Message::TrackEnded => {
                            self.player.unset_current();
                        }
                    };
                }
                recv(self.input_rx) -> event => {
                    self.handle_event(event.unwrap());
                }
            };

            if self.should_exit() {
                break;
            }
        }
    }

    pub fn handle_event(&mut self, ev: Event) {
        match ev {
            Event::Key(k) => {
                let keycode = k.code;
                let _modefier = k.modifiers;

                match keycode {
                    KeyCode::Esc => self.should_exit = true,
                    KeyCode::Char('k') => {
                        self.player
                            .set_cursor(self.player.cursor().saturating_sub(1));
                    }
                    KeyCode::Char('j') => {
                        let new_y = self.player.cursor() + 1;
                        if self.player.tracks_len() as u16 > new_y {
                            self.player.set_cursor(new_y);
                        }
                    }
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
        let [header, main] = Layout::new(
            Direction::Vertical,
            [Constraint::Length(2), Constraint::Length(4)],
        )
        .areas(area);

        let head = Line::raw(self.player.get_current().unwrap_or("No track"));

        head.render(header, buf);

        self.player.render(main, buf);
    }
}
