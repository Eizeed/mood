use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    style::Color,
    text::{Line, Text},
    widgets::Widget,
};
use rusqlite::Connection;

use crate::{
    action::Action,
    app::Mode,
    model::{self, playlist::PlaylistMd},
    widget::popup::{self, Popup},
};

pub struct Playlist {
    pub list: Vec<model::PlaylistMd>,

    pub popup: Popup,
    pub selected_track: Option<model::Track>,

    pub focused_widget: Focus,

    pub cursor: u16,
    pub show_cursor: bool,

    pub y_offset: u16,

    pub area: Rect,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    FocusTracklist,
    SetPlaylist(PlaylistMd),
    AddTrackToPlaylist(PlaylistMd, model::Track),
    DeletePlaylist(PlaylistMd),
    SetMode(Mode),
    CreatePlaylist(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    Popup(popup::Message),

    SetPlaylists(Vec<model::PlaylistMd>),

    FocusTracklist,

    SelectPlaylist,

    SetTrack(model::Track),
    AddSelectedToPlaylist,
    DeletePlaylist,
    CreatePlaylist,
    OpenPopup,

    CursorDown(u16),
    CursorUp(u16),

    Resize(Rect),
}

#[derive(Debug, Clone)]
pub enum Focus {
    Parent,
    Popup,
}

impl Playlist {
    pub fn new(conn: &Connection, area: Rect) -> Self {
        let list = PlaylistMd::get_all(conn);
        Playlist {
            list,
            popup: Popup::new(),
            focused_widget: Focus::Parent,
            selected_track: None,
            cursor: 0,
            show_cursor: true,
            y_offset: 0,
            area,
        }
    }

    pub fn handle_input(&self, code: KeyCode, mods: KeyModifiers) -> Option<Message> {
        match self.focused_widget {
            Focus::Parent => {
                let message = match code {
                    KeyCode::Enter => {
                        if self.selected_track.is_some() {
                            Message::AddSelectedToPlaylist
                        } else {
                            Message::SelectPlaylist
                        }
                    }
                    KeyCode::Char('j') => Message::CursorDown(1),
                    KeyCode::Char('k') => Message::CursorUp(1),
                    KeyCode::Char('p') => Message::FocusTracklist,
                    KeyCode::Char('d') => Message::DeletePlaylist,
                    KeyCode::Char('c') => Message::OpenPopup,
                    _ => return None,
                };

                Some(message)
            }
            Focus::Popup => {
                let message = match code {
                    KeyCode::End => Message::CreatePlaylist,
                    _ => self.popup.handle_input(code, mods).map(Message::Popup)?,
                };

                Some(message)
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::Popup(message) => {
                let mut action = Action::none();
                action.message = self.popup.update(message).map(Message::Popup);
                action
            }

            Message::SetPlaylists(playlists) => {
                if self.list.len() > playlists.len() {
                    let diff = (self.list.len() - playlists.len()) as u16;
                    let left = diff.saturating_sub(self.y_offset);
                    self.y_offset = self.y_offset.saturating_sub(diff);
                    self.cursor = self.cursor.saturating_sub(left);
                }
                self.list = playlists;
                Action::none()
            }
            Message::FocusTracklist => Action::instruction(Instruction::FocusTracklist),
            Message::SelectPlaylist => {
                if let Some(playlist_md) = self.get_under_cursor() {
                    Action::instruction(Instruction::SetPlaylist(playlist_md))
                } else {
                    Action::none()
                }
            }
            Message::SetTrack(track) => {
                self.selected_track = Some(track);
                Action::none()
            }
            Message::AddSelectedToPlaylist => {
                let selected = self
                    .selected_track
                    .take()
                    .expect("Selected track is expected to be some");

                if let Some(playlist_md) = self.get_under_cursor() {
                    Action::instruction(Instruction::AddTrackToPlaylist(playlist_md, selected))
                } else {
                    Action::none()
                }
            }
            Message::DeletePlaylist => {
                if let Some(playlist_md) = self.get_under_cursor() {
                    self.list.retain(|p| p.uuid != playlist_md.uuid);
                    Action::instruction(Instruction::DeletePlaylist(playlist_md))
                } else {
                    Action::none()
                }
            }
            Message::CreatePlaylist => {
                self.focused_widget = Focus::Parent;
                let name = std::mem::take(&mut self.popup.buffer);
                Action::instruction(Instruction::CreatePlaylist(name))
            }
            Message::OpenPopup => {
                self.focused_widget = Focus::Popup;
                Action::instruction(Instruction::SetMode(Mode::Write))
            }

            Message::CursorDown(count) => {
                self.cursor_down(count);
                Action::none()
            }
            Message::CursorUp(count) => {
                self.cursor_up(count);
                Action::none()
            }

            Message::Resize(area) => {
                self.resize(area);
                Action::none()
            }
        }
    }

    fn get_under_cursor(&self) -> Option<model::PlaylistMd> {
        let index = (self.cursor + self.y_offset) as usize;

        self.list.get(index).cloned()
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
        let total = self.list.len() as u16;

        if self.cursor + count < self.area.height && self.y_offset + self.cursor + count < total
        {
            self.cursor += count;
        } else if self.y_offset + self.area.height - 1 < total - 1 {
            self.y_offset += 1;
        }
    }

    fn resize(&mut self, area: Rect) {
        self.area = area;
    }
}

impl Widget for &Playlist {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let w = area.width;
        let y = self.cursor + area.y;

        for x in 0..w {
            buf.cell_mut((x, y)).unwrap().set_fg(Color::Green);
        }

        let list = Text::from_iter(
            self.list
                .iter()
                .skip(self.y_offset as usize)
                .map(|t| Line::raw(&t.name)),
        );

        list.render(area, buf);

        match &self.focused_widget {
            Focus::Popup => {
                let h = 3;
                let w = self.area.width;
                let x = self.area.x;
                let y = self.area.height / 2 - 2 + self.area.y;
                let area = Rect::new(x, y, w, h);
                self.popup.render(area, buf);
            }
            Focus::Parent => {}
        }
    }
}
