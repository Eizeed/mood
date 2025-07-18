use std::collections::VecDeque;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::{
    app::{Context, Repeat},
    model::{self, Track},
    widget::{ControlBar, Header, Playlist, Tracklist},
};

#[derive(Debug)]
pub struct Player {
    pub header: Header,
    pub tracklist: Tracklist,
    pub playlist: Playlist,
    pub control_bar: ControlBar,

    // pub selected_playlist: Option<Vec<Track>>,
    pub from_auto: bool,

    pub focused_widget: Focus,

    pub area: Rect,
}

#[derive(Debug)]
pub enum Focus {
    Tracklist,
    Playlist,
}

impl Player {
    pub fn new(tracklist: Vec<Track>, playlists: Vec<model::Playlist>, area: Rect) -> Self {
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
            tracklist: Tracklist::new(tracklist, playlist_area),
            playlist: Playlist::new(playlists, playlist_area),
            control_bar: ControlBar::new(control_area),

            from_auto: true,

            focused_widget: Focus::Tracklist,
            area,
        }
    }

    pub fn switch_window(&mut self) {
        match self.focused_widget {
            Focus::Tracklist => self.focused_widget = Focus::Playlist,
            Focus::Playlist => self.focused_widget = Focus::Tracklist,
        }
    }

    pub fn push_front_manual_queue(&mut self, track: Track) {
        self.tracklist.manual_queue.push_back(track);
    }

    pub fn set_auto_queue(&mut self, index: usize) {
        let mut list = self.tracklist.list.clone();

        let after = list.split_off(index);

        self.tracklist.history = list;
        self.tracklist.auto_queue = after.into();
    }

    pub fn get_current(&self) -> Option<&Track> {
        self.tracklist.current_track.as_ref()
    }

    pub fn take_current(&mut self) -> Option<Track> {
        self.tracklist.current_track.take()
    }

    pub fn set_current(&mut self, current: Track) {
        let name = current.path.file_stem().unwrap();

        self.control_bar.name = name.to_string_lossy().to_string();
        self.tracklist.current_track = Some(current);
    }

    pub fn unset_current(&mut self) {
        self.control_bar.name = "".to_string();
        self.tracklist.current_track = None;
    }

    pub fn get_under_cursor(&self) -> Track {
        self.tracklist.get_under_cursor()
    }

    pub fn pop_auto_queue(&mut self) -> Option<Track> {
        self.tracklist.auto_queue.pop_front()
    }

    pub fn manual_queue_mut(&mut self) -> &mut VecDeque<Track> {
        &mut self.tracklist.manual_queue
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
                    self.tracklist.history.push(current);

                    track
                }
                Repeat::Queue | Repeat::One => {
                    let current = self.take_current()?;
                    self.tracklist.history.push(current);

                    let track = match self.pop_auto_queue() {
                        Some(track) => track,
                        None => {
                            self.tracklist.auto_queue = self.tracklist.list.clone().into();

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
        if self.tracklist.history.is_empty() {
            return None;
        }

        if self.from_auto {
            let current = self.take_current()?;
            self.tracklist.auto_queue.push_front(current);
        }

        let track = self.tracklist.history.pop().unwrap();

        Some(track)
    }

    pub fn cursor_up(&mut self, count: u16) {
        match self.focused_widget {
            Focus::Tracklist => self.tracklist.cursor_up(count),
            Focus::Playlist => self.playlist.cursor_up(count),
        }
    }

    pub fn cursor_down(&mut self, count: u16) {
        match self.focused_widget {
            Focus::Tracklist => self.tracklist.cursor_down(count),
            Focus::Playlist => self.playlist.cursor_down(count),
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
        self.tracklist.resize(playlist_area);
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

        match self.focused_widget {
            Focus::Tracklist => self.tracklist.render(main_area, buf),
            Focus::Playlist => self.playlist.render(main_area, buf),
        }

        self.control_bar.render(control_area, buf, state);
    }
}
