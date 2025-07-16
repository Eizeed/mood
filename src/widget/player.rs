use std::collections::VecDeque;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::{
    app::{Context, Repeat},
    widget::{
        ControlBar, Header, Playlist,
        playlist::Track,
    },
};

pub struct Player {
    pub header: Header,
    pub playlist: Playlist,
    pub control_bar: ControlBar,

    pub from_auto: bool,

    pub focused_widget: Focus,

    pub area: Rect,
}

pub enum Focus {
    Playlist,
}

impl Player {
    pub fn new(playlist: Vec<Track>, area: Rect) -> Self {
        let [header_area, playlist_area, control_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(Header::HEIGHT),
                Constraint::Fill(1),
                Constraint::Length(ControlBar::HEIGHT),
            ],
        )
        .areas(area);

        Player {
            header: Header::new("HEADER".to_string(), header_area),
            playlist: Playlist::new(playlist, playlist_area),
            control_bar: ControlBar::new(control_area),

            from_auto: true,

            focused_widget: Focus::Playlist,
            area,
        }
    }

    pub fn push_front_manual_queue(&mut self, track: Track) {
        self.playlist.manual_queue.push_back(track);
    }

    pub fn set_auto_queue(&mut self, index: usize) {
        let mut list = self.playlist.list.clone();

        let after = list.split_off(index);

        self.playlist.history = list;
        self.playlist.auto_queue = after.into();
    }

    pub fn get_current(&self) -> Option<&Track> {
        self.playlist.current_track.as_ref()
    }

    pub fn take_current(&mut self) -> Option<Track> {
        self.playlist.current_track.take()
    }

    pub fn set_current(&mut self, current: Track) {
        let name = current.path.file_stem().unwrap();

        self.control_bar.name = name.to_string_lossy().to_string();
        self.playlist.current_track = Some(current);
    }

    pub fn unset_current(&mut self) {
        self.control_bar.name = "".to_string();
        self.playlist.current_track = None;
    }

    pub fn get_under_cursor(&self) -> Track {
        self.playlist.get_under_cursor()
    }

    pub fn pop_auto_queue(&mut self) -> Option<Track> {
        self.playlist.auto_queue.pop_front()
    }

    pub fn manual_queue_mut(&mut self) -> &mut VecDeque<Track> {
        &mut self.playlist.manual_queue
    }

    pub fn get_next(&mut self, repeat: Repeat) -> Option<Track> {
        let track = if self.manual_queue_mut().is_empty() {
            match repeat {
                Repeat::None => {
                    let track = match self.pop_auto_queue() {
                        Some(track) => track,
                        None => return None,
                    };

                    let current = self.take_current()?;
                    self.playlist.history.push(current);

                    track
                }
                Repeat::Queue | Repeat::One => {
                    let current = self.take_current()?;
                    self.playlist.history.push(current);

                    let track = match self.pop_auto_queue() {
                        Some(track) => track,
                        None => {
                            self.playlist.auto_queue = self.playlist.list.clone().into();

                            self.pop_auto_queue().unwrap()
                        }
                    };

                    track
                }
            }
        } else {
            let next_track = self.manual_queue_mut().pop_front().unwrap();

            next_track
        };

        Some(track)
    }

    pub fn get_prev(&mut self) -> Option<Track> {
        if self.playlist.history.is_empty() {
            return None;
        }

        if self.from_auto {
            let current = self.take_current()?;
            self.playlist.auto_queue.push_front(current);
        }

        let track = self.playlist.history.pop().unwrap();

        Some(track)
    }

    pub fn cursor_up(&mut self, count: u16) {
        match self.focused_widget {
            Focus::Playlist => {
                self.playlist.cursor_up(count);
            }
        }
    }

    pub fn cursor_down(&mut self, count: u16) {
        match self.focused_widget {
            Focus::Playlist => {
                self.playlist.cursor_down(count);
            }
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let area = Rect::new(0, 0, width, height);
        let [header_area, playlist_area, control_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(Header::HEIGHT),
                Constraint::Fill(1),
                Constraint::Length(ControlBar::HEIGHT),
            ],
        )
        .areas(area);

        self.header.resize(header_area);
        self.playlist.resize(playlist_area);
        self.control_bar.resize(control_area);
    }
}

impl StatefulWidget for &Player {
    type State = Context;

    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let [header_area, main_area, control_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(Header::HEIGHT),
                Constraint::Fill(1),
                Constraint::Length(ControlBar::HEIGHT),
            ],
        )
        .areas(area);

        self.header.render(header_area, buf);
        self.playlist.render(main_area, buf);
        self.control_bar.render(control_area, buf, state);
    }
}
