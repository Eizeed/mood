use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Rect},
    style::Color,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use rusqlite::Connection;

use crate::{
    action::Action,
    app::Mode,
    model::{self, playlist::PlaylistMd},
    task::Task,
    widget::{
        Component,
        gen_popup::{self, Popup},
        new_playlist_input::{self, NewPlaylistInput},
    },
};

pub struct Playlist {
    pub list: Vec<model::PlaylistMd>,

    pub popup: gen_popup::Popup<NewPlaylistInput>,
    pub selected_track: Option<model::Track>,

    pub focused_widget: Focus,

    pub cursor: u16,
    // pub show_cursor: bool,
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
    Popup(new_playlist_input::Message),

    SetPlaylists(Vec<model::PlaylistMd>),

    FocusTracklist,

    SelectPlaylist,

    SetTrack(model::Track),
    AddSelectedToPlaylist,
    DeletePlaylist,
    CreatePlaylist(String),
    OpenPopup,
    ClosePopup,

    CursorDown(u16),
    CursorUp(u16),

    Resize(Rect),
}

#[derive(Debug, Clone)]
pub enum Focus {
    Parent,
    Popup,
}

#[derive(Debug, Clone)]
enum PlaylistInstruction {
    Popup(new_playlist_input::Instruction),
}

impl Playlist {
    pub fn new(conn: &Connection, area: Rect) -> Self {
        let list = PlaylistMd::get_all(conn);
        Playlist {
            list,
            popup: Popup::new(NewPlaylistInput::new(
                area.centered(Constraint::Percentage(75), Constraint::Length(3)),
            )),
            focused_widget: Focus::Parent,
            selected_track: None,
            cursor: 0,
            // show_cursor: true,
            y_offset: 0,
            area,
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
            self.cursor = 0;
        } else {
            self.cursor -= count;
        }
    }

    fn cursor_down(&mut self, count: u16) {
        let total = self.list.len() as u16;

        if self.cursor + count <= self.area.height - 1
            && self.y_offset + self.cursor + count < total
        {
            self.cursor += count;
        } else if self.y_offset + self.area.height - 1 < total {
            self.y_offset += 1;
        }
    }

    fn align_cursor(&mut self) {
        if !self.list.is_empty() && self.cursor + self.y_offset >= self.list.len() as u16 - 1 {
            self.cursor = self.list.len() as u16 - self.y_offset - 1;
        }
    }

    fn perform(&mut self, instruction: PlaylistInstruction) -> Task<Message> {
        match instruction {
            PlaylistInstruction::Popup(popup_inst) => match popup_inst {
                new_playlist_input::Instruction::Submit(name) => {
                    Task::new(Message::CreatePlaylist(name))
                }
                new_playlist_input::Instruction::Cancel => Task::new(Message::ClosePopup),
            },
        }
    }
}

impl Component for Playlist {
    type Message = Message;
    type Output = Action<Instruction, Message>;

    fn area(&self) -> Rect {
        self.area
    }

    fn resize(&mut self, area: Rect) {
        self.area = area;
        if self.cursor > area.height - 1 {
            self.cursor = area.height - 1;
        }
    }

    fn update(&mut self, message: Self::Message) -> Self::Output {
        match message {
            Message::Popup(message) => {
                let action = self
                    .popup
                    .update(message)
                    .map(Message::Popup)
                    .map_instruction(PlaylistInstruction::Popup);

                let task = if let Some(instruction) = action.instruction {
                    self.perform(instruction)
                } else {
                    Task::none()
                };

                Action::task(action.task.extend(task))
            }

            Message::SetPlaylists(playlists) => {
                self.list = playlists;
                self.align_cursor();
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
            Message::CreatePlaylist(playlist_title) => {
                self.focused_widget = Focus::Parent;
                self.popup.hide();

                Action::instruction(Instruction::CreatePlaylist(playlist_title))
            }
            Message::OpenPopup => {
                self.focused_widget = Focus::Popup;
                self.popup.show();
                Action::instruction(Instruction::SetMode(Mode::Write))
            }
            Message::ClosePopup => {
                self.focused_widget = Focus::Parent;
                self.popup.hide();
                Action::instruction(Instruction::SetMode(Mode::Default))
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
                self.popup
                    .resize(area.centered_vertically(ratatui::layout::Constraint::Length(3)));
                Action::none()
            }
        }
    }

    fn handle_input(&self, code: KeyCode, mods: KeyModifiers) -> Option<Message> {
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
                let message = self.popup.handle_input(code, mods).map(Message::Popup)?;

                Some(message)
            }
        }
    }

    fn view(&self, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let area = self.area();

        if self.list.is_empty() && matches!(self.focused_widget, Focus::Parent) {
            let mut center_area = area;
            center_area.y = (area.y + area.height / 2) - 1;
            center_area.height = 3;

            let width = area.width / 3;
            center_area.x = width;
            center_area.width = width;

            Paragraph::new("No playlists")
                .centered()
                .block(Block::bordered())
                .render(center_area, buf);

            return;
        }

        let list = Text::from_iter(
            self.list
                .iter()
                .skip(self.y_offset as usize)
                .map(|t| Line::raw(&t.name)),
        );

        let w = area.width;
        let y = self.cursor + area.y;

        for x in 0..w {
            buf.cell_mut((x, y)).unwrap().set_fg(Color::Green);
        }
        list.render(area, buf);

        match &self.focused_widget {
            Focus::Popup => {
                self.popup.view(buf);
            }
            Focus::Parent => {}
        }
    }
}
