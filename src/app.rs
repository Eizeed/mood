use std::sync::mpsc;

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::Widget,
};

use crate::{
    music::{Command, spawn_music},
    player::Player,
};

pub struct App {
    player: Player,
    pub should_exit: bool,
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

    pub fn handle_event(&mut self, ev: Event) {
        match ev {
            Event::Key(k) => {
                let keycode = k.code;
                let _modefier = k.modifiers;

                match keycode {
                    KeyCode::Esc => self.should_exit = true,
                    KeyCode::Char('k') => {
                        self.player.cursor.y = self.player.cursor.y.saturating_sub(1);
                    }
                    KeyCode::Char('j') => {
                        if self.player.tracks.len() as u16 > self.player.cursor.y + 1 {
                            self.player.cursor.y += 1;
                        }
                    }
                    KeyCode::Enter => {
                        self.player.current =
                            Some(self.player.tracks[self.player.cursor.y as usize].clone());

                        self.music_handle
                            .send(Command::Play(self.player.current.as_ref().unwrap().into()))
                            .unwrap();
                    }
                    KeyCode::Char(' ') => {
                        if self.player.is_paused {
                            self.music_handle.send(Command::Resume).unwrap();
                            self.player.is_paused = false;
                        } else {
                            self.music_handle.send(Command::Pause).unwrap();
                            self.player.is_paused = true;
                        }
                    }
                    _ => {}
                }
            }
            Event::Mouse(m) => {}
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
            [Constraint::Length(1), Constraint::Length(4)],
        )
        .areas(area);

        let head = Line::raw(
            self.player
                .current
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("No track"),
        );

        head.render(header, buf);

        self.player.render(main, buf);
    }
}
