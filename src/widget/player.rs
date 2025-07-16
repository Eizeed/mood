use std::{collections::VecDeque, path::PathBuf};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::{
    app::{Context, Repeat},
    widget::{
        ControlBar, Header, Playlist,
        playlist::{CurrentTrack, Track},
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
    pub fn new(playlist: Vec<PathBuf>, area: Rect) -> Self {
        let [_header_area, playlist_area, control_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(4),
            ],
        )
        .areas(area);

        Player {
            header: Header {
                msg: "Hello world".to_string(),
            },
            playlist: Playlist::new(playlist, playlist_area),
            control_bar: ControlBar::new(control_area),

            from_auto: true,

            focused_widget: Focus::Playlist,
            area,
        }
    }

    // pub fn change_playlist(&mut self, playlist: Vec<PathBuf>) {}

    pub fn push_front_manual_queue(&mut self, track: Track) {
        self.playlist.manual_queue.push_back(track);
    }

    pub fn set_auto_queue(&mut self, index: usize) {
        let mut list = self
            .playlist
            .list
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, path)| Track { index: i, path })
            .collect::<Vec<Track>>();

        let after = list.split_off(index);

        self.playlist.history = list;
        self.playlist.auto_queue = after.into();
    }

    pub fn get_current(&self) -> Option<&CurrentTrack> {
        self.playlist.current_track.as_ref()
    }

    pub fn take_current(&mut self) -> Option<CurrentTrack> {
        self.playlist.current_track.take()
    }

    pub fn set_current(&mut self, current: CurrentTrack) {
        let name = current.path.file_stem().unwrap();

        self.control_bar.name = name.to_string_lossy().to_string();
        self.playlist.current_track = Some(current);
    }

    pub fn unset_current(&mut self) {
        self.control_bar.name = "".to_string();
        self.playlist.current_track = None;
    }

    pub fn toggle_repeat() {}
    pub fn toggle_shuffle() {}
    pub fn toggle_pause() {}

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
                    self.playlist.history.push(Track {
                        index: current.index,
                        path: current.path,
                    });

                    track
                }
                Repeat::Queue | Repeat::One => {
                    let current = self.take_current()?;
                    self.playlist.history.push(Track {
                        index: current.index,
                        path: current.path,
                    });

                    let track = match self.pop_auto_queue() {
                        Some(track) => track,
                        None => {
                            self.playlist.auto_queue = self
                                .playlist
                                .list
                                .clone()
                                .into_iter()
                                .enumerate()
                                .map(|(idx, path)| Track { index: idx, path })
                                .collect();

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
            self.playlist.history.push(Track {
                index: current.index,
                path: current.path,
            });
        }

        let track = self.playlist.history.pop().unwrap();

        Some(track)
    }

    pub fn seek_forward() {}
    pub fn seek_backward() {}

    pub fn cursor_up(&mut self, count: u16) {
        match self.focused_widget {
            Focus::Playlist => {
                let playlist = &mut self.playlist;

                let count = count as u16;
                if playlist.cursor < count {
                    let rest = count - playlist.cursor;
                    playlist.y_offset = playlist.y_offset.saturating_sub(rest);
                } else {
                    playlist.cursor -= count;
                }
            }
        }
    }

    pub fn cursor_down(&mut self, count: u16) {
        match self.focused_widget {
            Focus::Playlist => {
                let playlist = &mut self.playlist;

                let total = playlist.list.len() as u16;

                if playlist.cursor + (count as u16) < playlist.area.height
                    && playlist.y_offset + playlist.cursor + (count as u16)
                        < playlist.list.len() as u16
                {
                    playlist.cursor += count as u16;
                } else if playlist.y_offset + playlist.area.height - 1 < total - 1 {
                    playlist.y_offset += 1;
                }
            }
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let area = Rect::new(0, 0, width, height);
        let [_header_area, playlist_area, control_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(4),
            ],
        )
        .areas(area);

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
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(4),
            ],
        )
        .areas(area);

        self.header.render(header_area, buf);
        self.playlist.render(main_area, buf);
        self.control_bar.render(control_area, buf, state);
    }
}
