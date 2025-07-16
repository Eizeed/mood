use std::{collections::VecDeque, path::PathBuf, time::Duration};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::Widget,
};

pub struct Playlist {
    pub list: Vec<PathBuf>,

    pub auto_queue: VecDeque<Track>,
    pub manual_queue: VecDeque<Track>,
    pub history: Vec<Track>,

    pub current_track: Option<CurrentTrack>,

    pub cursor: u16,
    pub show_cursor: bool,

    pub y_offset: u16,

    pub area: Rect,
}

#[derive(Debug)]
pub struct Track {
    pub index: usize,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct CurrentTrack {
    pub index: usize,
    pub path: PathBuf,
    pub duration: Duration,
}

impl Playlist {
    pub fn new(paths: Vec<PathBuf>, area: Rect) -> Self {
        Playlist {
            list: paths,
            auto_queue: VecDeque::new(),
            manual_queue: VecDeque::new(),
            history: Vec::new(),
            current_track: None,
            cursor: 0,
            show_cursor: true,
            y_offset: 0,
            area,
        }
    }

    pub fn get_under_cursor(&self) -> Track {
        let index = (self.cursor + self.y_offset) as usize;
        assert!(index < self.list.len(), "Index of cursor is out of bounds");

        let path = self.list[index].clone();

        Track { index, path }
    }

    pub fn cursor_up(&mut self, count: u16) {
        if self.cursor < count {
            let rest = count - self.cursor;
            self.y_offset = self.y_offset.saturating_sub(rest);
        } else {
            self.cursor -= count;
        }
    }

    pub fn cursor_down(&mut self, count: u16) {
        let total = self.list.len() as u16;

        if self.cursor + (count as u16) < self.area.height
            && self.y_offset + self.cursor + (count as u16) < self.list.len() as u16
        {
            self.cursor += count as u16;
        } else if self.y_offset + self.area.height - 1 < total - 1 {
            self.y_offset += 1;
        }
    }

    pub fn resize(&mut self, area: Rect) {
        self.area = area;
    }
}

impl Widget for &Playlist {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let w = area.width;
        let y = self.cursor + area.y;
        for x in 0..w {
            buf.cell_mut((x, y)).unwrap().set_fg(Color::Green);
        }

        let current = self.current_track.as_ref();
        let list =
            match current {
                Some(current) => Text::from_iter(
                    self.list
                        .iter()
                        .skip(self.y_offset as usize)
                        .enumerate()
                        .map(|(i, t)| {
                            let path = t.to_string_lossy();
                            let name = path.split("/").last().unwrap().to_string();

                            // NOTE: Idk what this is doing (i wrote it)
                            // spend some time in future to understand
                            let line = if current.path.to_string_lossy().contains(&name)
                                && current.index - self.y_offset as usize == i
                            {
                                let color = if self.cursor + self.y_offset == current.index as u16 {
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
                None => Text::from_iter(self.list.iter().skip(self.y_offset as usize).map(|t| {
                    Line::raw(t.to_string_lossy().split("/").last().unwrap().to_string())
                })),
            };

        list.render(area, buf);
    }
}
