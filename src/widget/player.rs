use std::{collections::VecDeque, path::Path, time::Duration};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::Widget,
};

use crate::io::get_files;

use super::Cursor;


// TODO: Brainstorm this bitch
pub struct Player {
    // Should be all_tracks
    // And need to create playlist
    // for current tracks, idk
    tracks: Vec<String>,
    current: Option<CurrentTrack>,

    // That's the main vector.
    // Tracks would be poped from here
    // if there is no tracks in manual queue
    // Poping because need to apply shuffle
    // and for now i don't know how to implement
    // it without poping played tracks. Sadge
    auto_queue: VecDeque<Track>,

    // Vector for user added tracks.
    // They have priority.
    // Repeat doesn't apply to them.
    // After being poped, track won't be
    // added to histry vector
    manual_queue: VecDeque<Track>,
    // history: Vec<Track>,
    is_paused: bool,

    cursor: Cursor,

    area: Rect,
    pub y_offset: u16,
}

impl Player {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        let tracks = get_files(path, "mp3");

        let names: Vec<String> = tracks
            .into_iter()
            .map(|pb| pb.to_str().unwrap().to_string())
            .collect();

        Player {
            tracks: names,
            current: None,
            auto_queue: VecDeque::new(),
            manual_queue: VecDeque::new(),
            cursor: Cursor::new(),
            is_paused: false,
            area: Rect::ZERO,
            y_offset: 0,
        }
    }

    pub fn cursor_up(&mut self, count: usize) {
        let count = count as u16;
        if self.cursor.y < count {
            let rest = count - self.cursor.y;
            self.y_offset = self.y_offset.saturating_sub(rest);
        } else {
            self.cursor.y -= count;
        }
    }

    pub fn cursor_down(&mut self, count: usize) {
        let total = self.tracks_len() as u16;
        if self.cursor.y + (count as u16) < self.area.height
            && self.cursor.y + self.y_offset < total
        {
            self.cursor.y += count as u16;
        } else if self.y_offset + self.area.height - 1 < total - 1 {
            self.y_offset += 1;
        }
    }

    pub fn manual_queue_mut(&mut self) -> &mut VecDeque<Track> {
        &mut self.manual_queue
    }

    pub fn auto_queue_mut(&mut self) -> &mut VecDeque<Track> {
        &mut self.auto_queue
    }

    pub fn set_auto_queue<T: Iterator<Item = Track>>(&mut self, tracks: T) {
        self.auto_queue = tracks.into_iter().collect();
    }

    pub fn tracks(&self) -> &[String] {
        &self.tracks
    }

    pub fn tracks_len(&self) -> usize {
        self.tracks.len()
    }

    pub fn track_under_cursor(&self) -> (String, usize) {
        let idx = (self.cursor.y + self.y_offset) as usize;
        (self.tracks[idx].clone(), idx)
    }

    pub fn get_current_path(&self) -> Option<&str> {
        self.current.as_ref().map(|s| s.path.as_str())
    }

    pub fn get_current_duration(&self) -> Option<Duration> {
        self.current.as_ref().map(|s| s.duration)
    }

    pub fn get_current_index(&self) -> Option<usize> {
        self.current.as_ref().map(|c| c.index)
    }

    pub fn get_current(&self) -> Option<&CurrentTrack> {
        self.current.as_ref()
    }

    pub fn set_current<T: Into<usize>>(&mut self, str: String, duration: Duration, index: T) {
        self.current = Some(CurrentTrack {
            path: str.into(),
            duration,
            index: index.into(),
        })
    }

    pub fn unset_current(&mut self) {
        self.current = None;
    }

    pub fn cursor(&self) -> u16 {
        self.cursor.y
    }

    pub fn set_cursor(&mut self, y: u16) {
        self.cursor.y = y;
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn set_is_paused(&mut self, is_paused: bool) {
        self.is_paused = is_paused;
    }
}

impl Widget for &mut Player {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.area = area;
        self.cursor.render(area, buf);

        let current = self.current.as_ref();
        let list = match current {
            Some(current) => Text::from_iter(
                self.tracks
                    .iter()
                    .skip(self.y_offset as usize)
                    .enumerate()
                    .map(|(i, t)| {
                        let name = t.split("/").last().unwrap();

                        let line = if current.path.contains(name)
                            && current.index - self.y_offset as usize == i
                        {
                            let color = if self.cursor() == current.index as u16 {
                                Color::Yellow
                            } else {
                                Color::Blue
                            };

                            Line::raw(name).fg(color)
                        } else {
                            Line::raw(name)
                        };

                        line
                    }),
            ),
            None => Text::from_iter(
                self.tracks
                    .iter()
                    .skip(self.y_offset as usize)
                    .map(|t| Line::raw(t.split("/").last().unwrap())),
            ),
        };

        list.render(area, buf);
    }
}

#[derive(Debug)]
pub struct CurrentTrack {
    pub path: String,
    pub duration: Duration,
    pub index: usize,
}

#[derive(Debug)]
pub struct Track {
    pub path: String,
    pub index: usize,
}
