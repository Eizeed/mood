use std::{
    collections::VecDeque,
    rc::Rc,
    time::{Duration, Instant},
};

use rand::seq::SliceRandom;
use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::Widget,
};

use crate::{
    action::Action,
    app::{Repeat, Shuffle},
    model,
};

#[derive(Debug)]
pub struct Tracklist {
    pub library: Rc<[model::Track]>,
    pub base: Rc<[model::Track]>,

    pub list: Vec<model::Track>,
    pub auto_queue: VecDeque<model::Track>,
    pub manual_queue: VecDeque<model::Track>,
    pub history: Vec<model::Track>,

    pub current_track: Option<model::Track>,
    pub from_auto: bool,

    pub selected_playlist: Option<model::Playlist>,

    pub shuffle: Shuffle,
    pub repeat: Repeat,

    pub start_timer: Instant,

    pub cursor: u16,
    pub show_cursor: bool,

    pub y_offset: u16,

    pub area: Rect,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Play(model::Track),
    FocusPlaylist,
    AddToPlaylist(model::Track),
    RemoveFromPlaylist(model::Playlist, model::Track),
    SetHeader(String),
    Exit,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetCurrent,
    QueueSelected,

    SetBaseQueue,

    SelectTrack,

    SkipToNext,
    SkipToPrev,

    CursorDown(u16),
    CursorUp(u16),

    FocusPlaylist,

    RemoveTrack,

    GetNext,

    SetPlaylist(model::Playlist),

    Resize(Rect),
}

impl Tracklist {
    pub fn new(tracks: Rc<[model::Track]>, area: Rect, shuffle: Shuffle, repeat: Repeat) -> Self {
        Tracklist {
            library: tracks.clone(),
            base: tracks.clone(),
            list: tracks.to_vec(),
            auto_queue: VecDeque::new(),
            manual_queue: VecDeque::new(),
            history: Vec::new(),
            current_track: None,
            from_auto: false,
            selected_playlist: None,
            start_timer: Instant::now(),
            shuffle,
            repeat,
            cursor: 0,
            show_cursor: true,
            y_offset: 0,
            area,
        }
    }

    pub fn handle_input(&self, code: KeyCode, mods: KeyModifiers) -> Option<Message> {
        let message = match code {
            KeyCode::Enter => Message::SetCurrent,
            KeyCode::Char('a') => Message::SelectTrack,
            KeyCode::Char('q') => Message::QueueSelected,
            KeyCode::Char('k') | KeyCode::Up => Message::CursorUp(1),
            KeyCode::Char('j') | KeyCode::Down => match mods {
                KeyModifiers::NONE => Message::CursorDown(1),
                _ => return None,
            },
            KeyCode::Char('p') => Message::FocusPlaylist,
            KeyCode::Char('d') => Message::RemoveTrack,
            _ => return None,
        };

        Some(message)
    }

    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::GetNext => {
                if self.start_timer.elapsed() < Duration::from_millis(50) {
                    return Action::none();
                }
                let Some(track) = self.get_next() else {
                    return Action::none();
                };

                self.start_timer = Instant::now();
                self.current_track = Some(track.clone());
                Action::instruction(Instruction::Play(track))
            }
            Message::SetBaseQueue => {
                self.base = self.library.clone();
                self.selected_playlist = None;
                Action::instruction(Instruction::SetHeader("".to_string()))
            }

            Message::SelectTrack => {
                if let Some(track) = self.get_under_cursor() {
                    Action::instruction(Instruction::AddToPlaylist(track))
                } else {
                    Action::none()
                }
            }

            Message::SetCurrent => {
                if self.start_timer.elapsed() < Duration::from_millis(50) {
                    return Action::none();
                }

                self.set_auto_queue((self.cursor + self.y_offset) as usize);
                let track = self.auto_queue.pop_front().unwrap();

                self.from_auto = true;
                self.current_track = Some(track.clone());
                self.start_timer = Instant::now();

                Action::instruction(Instruction::Play(track))
            }
            Message::QueueSelected => {
                if let Some(track) = self.get_under_cursor() {
                    self.manual_queue.push_back(track);
                };
                Action::none()
            }
            Message::CursorUp(amount) => {
                self.cursor_up(amount);
                Action::none()
            }
            Message::CursorDown(amount) => {
                self.cursor_down(amount);
                Action::none()
            }
            Message::SkipToNext => {
                if self.start_timer.elapsed() < Duration::from_millis(50) {
                    return Action::none();
                }

                let track = match self.get_next() {
                    Some(track) => track,
                    None => {
                        self.set_auto_queue(0);
                        let Some(track) = self.get_next() else {
                            return Action::none();
                        };

                        track
                    }
                };

                self.current_track = Some(track.clone());
                self.start_timer = Instant::now();

                Action::instruction(Instruction::Play(track))
            }
            Message::SkipToPrev => {
                if self.start_timer.elapsed() < Duration::from_millis(50) {
                    return Action::none();
                }

                let track = match self.get_prev() {
                    Some(track) => track,
                    None => {
                        self.history = self.list.clone();

                        let Some(track) = self.get_prev() else {
                            return Action::none();
                        };

                        track
                    }
                };

                self.current_track = Some(track.clone());
                self.start_timer = Instant::now();
                Action::instruction(Instruction::Play(track))
            }
            Message::FocusPlaylist => Action::instruction(Instruction::FocusPlaylist),
            Message::RemoveTrack => {
                if let Some(playlist) = self.selected_playlist.take() {
                    let index = (self.cursor + self.y_offset) as usize;
                    let track = playlist.tracks[index].clone();
                    Action::instruction(Instruction::RemoveFromPlaylist(playlist, track))
                } else {
                    Action::none()
                }
            }
            Message::SetPlaylist(playlist) => {
                self.selected_playlist = Some(playlist);
                self.cursor = 0;
                self.y_offset = 0;
                Action::none()
            }
            Message::Resize(area) => {
                self.area = area;
                Action::none()
            }
        }
    }

    pub fn get_current(&self) -> Option<model::Track> {
        self.current_track.clone()
    }

    fn get_under_cursor(&self) -> Option<model::Track> {
        let index = (self.cursor + self.y_offset) as usize;

        match &self.selected_playlist {
            Some(playlist) => playlist.tracks.get(index).cloned(),
            None => self.base.get(index).cloned(),
        }
    }

    fn cursor_up(&mut self, count: u16) {
        if self.cursor < count {
            let rest = count - self.cursor;
            self.y_offset = self.y_offset.saturating_sub(rest);
        } else {
            self.cursor -= count;
        }
    }

    fn cursor_down(&mut self, count: u16) {
        let total = self
            .selected_playlist
            .as_ref()
            .map(|playlist| playlist.tracks.len() as u16)
            .unwrap_or(self.base.len() as u16);

        if self.cursor + (count as u16) < self.area.height
            && self.y_offset + self.cursor + (count as u16) < total
        {
            self.cursor += count as u16;
        } else if self.y_offset + self.area.height - 1 < total - 1 {
            self.y_offset += 1;
        }
    }

    fn set_auto_queue(&mut self, index: usize) {
        if let Some(playlist) = &self.selected_playlist {
            self.base = playlist.tracks.clone().into();
        } else {
            self.base = self.library.clone();
        };

        let index = match self.shuffle {
            Shuffle::Random => {
                self.list = self.base.to_vec();

                self.list.swap(index, 0);
                self.list[1..].shuffle(&mut rand::rng());
                0
            }
            _ => {
                self.list = self.base.to_vec();
                index
            }
        };

        let mut list = self.list.clone();

        let after = list.split_off(index);

        self.history = list;
        self.auto_queue = after.into();
    }

    fn get_next(&mut self) -> Option<model::Track> {
        let track = if self.manual_queue.is_empty() {
            match self.repeat {
                Repeat::None => {
                    let track = self.auto_queue.pop_front()?;
                    let current = self.current_track.take()?;

                    if self.from_auto {
                        self.history.push(current);
                    }
                    self.from_auto = true;

                    track
                }
                Repeat::Queue | Repeat::One => {
                    let current = self.current_track.take()?;
                    if self.from_auto {
                        self.history.push(current);
                    }
                    self.from_auto = true;

                    let track = match self.auto_queue.pop_front() {
                        Some(track) => track,
                        None => {
                            self.auto_queue = self.list.clone().into();

                            self.auto_queue.pop_front().unwrap()
                        }
                    };

                    self.from_auto = true;
                    track
                }
            }
        } else {
            if self.from_auto {
                self.current_track.take().map(|t| self.history.push(t));
            }

            self.from_auto = false;
            let next_track = self.manual_queue.pop_front().unwrap();

            next_track
        };

        Some(track)
    }

    pub fn get_prev(&mut self) -> Option<model::Track> {
        if self.history.is_empty() {
            return None;
        }

        if self.from_auto {
            let current = self.current_track.take()?;
            self.auto_queue.push_front(current);
        }

        self.from_auto = true;

        let track = self.history.pop().unwrap();

        Some(track)
    }
}

impl Widget for &Tracklist {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let w = area.width;
        let y = self.cursor + area.y;
        for x in 0..w {
            buf.cell_mut((x, y)).unwrap().set_fg(Color::Green);
        }

        let current = self.current_track.as_ref();
        let list: &[model::Track] = self
            .selected_playlist
            .as_ref()
            .map(|playlist| &*playlist.tracks)
            .unwrap_or(&*self.base);

        let list = match current {
            Some(current) => Text::from_iter(
                list.into_iter()
                    .skip(self.y_offset as usize)
                    .enumerate()
                    .map(|(i, t)| {
                        let i = i + self.y_offset as usize;
                        let path = t.path.to_string_lossy();
                        let name = path.split("/").last().unwrap().to_string();

                        // NOTE: Idk what this is doing (i wrote it)
                        // spend some time in future to understand
                        let line = if current.path.to_string_lossy().contains(&name)
                            && (self.y_offset..self.y_offset + self.area.height)
                                .contains(&(i as u16))
                        {
                            let color = if list[(self.cursor + self.y_offset) as usize].uuid
                                == current.uuid
                            {
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
            None => Text::from_iter(list.into_iter().skip(self.y_offset as usize).map(|t| {
                Line::raw(
                    t.path
                        .to_string_lossy()
                        .split("/")
                        .last()
                        .unwrap()
                        .to_string(),
                )
            })),
        };

        list.render(area, buf);
    }
}
