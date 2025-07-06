use std::sync::mpsc;

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::Widget,
};

use crate::{
    music::{Command, spawn_music},
    widget::Player,
};

pub struct App {
    player: Player,
    should_exit: bool,
    music_handle: mpsc::Sender<Command>,
}

impl App {
    pub fn with_player(player: Player) -> Self {
        let (tx, rx) = mpsc::channel::<Command>();
        spawn_music(rx);

        App {
            player,
            music_handle: tx,
            should_exit: false,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
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

                        self.music_handle
                            .send(Command::play(self.player.get_current().unwrap()))
                            .unwrap();
                    }
                    KeyCode::Char(' ') => {
                        if self.player.is_paused() {
                            self.music_handle.send(Command::pause()).unwrap();
                            self.player.set_is_paused(false);
                        } else {
                            self.music_handle.send(Command::resume()).unwrap();
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
