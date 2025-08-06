use std::{ops::Range, time::Duration};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{MouseEvent, MouseEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Block, Borders, Widget},
};

use crate::{
    action::Action,
    app::{Repeat, Shuffle},
    widget::Component,
};

#[derive(Debug)]
pub struct ControlBar {
    pub name: String,
    // TODO: in future when i will bring more styles
    // Create a tracker for duration something like
    // 0:52---3:22
    // Or somthing simmilar

    // duration
    // pos
    pub progress: f32,
    pub repeat: Repeat,
    pub shuffle: Shuffle,
    pub paused: bool,

    pub control_bar_y: u16,

    pub shuffle_pos: Range<u16>,
    pub seek_backward_pos: Range<u16>,
    pub pause_pos: Range<u16>,
    pub seek_forward_pos: Range<u16>,
    pub repeat_pos: Range<u16>,
    pub area: Rect,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    SetShuffle(Shuffle),
    SetRepeat(Repeat),
    SetPause(bool),
    SeekForward(Duration),
    SeekBackward(Duration),
}

#[derive(Debug, Clone)]
pub enum Message {
    SetProgress(f32),
    SetName(String),

    ToggleShuffle,
    ToggleRepeat,
    TogglePause,
    SeekForward(Duration),
    SeekBackward(Duration),

    SetShuffle(Shuffle),
    SetRepeat(Repeat),
    SetPause(bool),

    Resize(Rect),
}

impl ControlBar {
    pub const HEIGHT: u16 = 4;

    pub fn new(full_area: Rect, progress: f32, shuffle: Shuffle, repeat: Repeat) -> Self {
        debug_assert_eq!(full_area.height, Self::HEIGHT);

        let [_, _, _, button_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        )
        .areas(full_area);

        let [_, area, _] = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Fill(1),
                Constraint::Length(19),
                Constraint::Fill(1),
            ],
        )
        .areas(button_area);

        let start = area.x;

        // bruh...
        let control_bar_y = area.y;
        let shuffle_pos = start..start + 3;
        let seek_backward_pos = start + 4..start + 7;
        let pause_pos = start + 8..start + 11;
        let seek_forward_pos = start + 12..start + 15;
        let repeat_pos = start + 16..start + 19;

        ControlBar {
            name: "".to_string(),

            progress,
            repeat,
            shuffle,
            paused: false,

            control_bar_y,

            shuffle_pos,
            seek_backward_pos,
            pause_pos,
            seek_forward_pos,
            repeat_pos,
            area: full_area,
        }
    }
}

impl Component for ControlBar {
    type Message = Message;
    type Output = Action<Instruction, Message>;

    fn area(&self) -> Rect {
        self.area
    }

    fn resize(&mut self, full_area: Rect) {
        debug_assert_eq!(full_area.height, Self::HEIGHT);
        let [_, _, _, button_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        )
        .areas(full_area);

        let [_, area, _] = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Fill(1),
                Constraint::Length(19),
                Constraint::Fill(1),
            ],
        )
        .areas(button_area);

        let start = area.x;

        // bruh...
        self.control_bar_y = area.y;
        self.shuffle_pos = start..start + 3;
        self.seek_backward_pos = start + 4..start + 7;
        self.pause_pos = start + 8..start + 11;
        self.seek_forward_pos = start + 12..start + 15;
        self.repeat_pos = start + 16..start + 19;
        self.area = full_area;
    }

    fn update(&mut self, message: Self::Message) -> Self::Output {
        match message {
            Message::SetProgress(progress) => {
                self.progress = progress;
            }
            Message::SetName(name) => {
                self.name = name;
            }
            Message::ToggleShuffle => {
                match self.shuffle {
                    Shuffle::None => self.shuffle = Shuffle::Random,
                    Shuffle::Random => self.shuffle = Shuffle::None,
                }

                return Action::instruction(Instruction::SetShuffle(self.shuffle.clone()));
            }
            Message::ToggleRepeat => {
                match self.repeat {
                    Repeat::None => self.repeat = Repeat::Queue,
                    Repeat::Queue => self.repeat = Repeat::One,
                    Repeat::One => self.repeat = Repeat::None,
                }

                return Action::instruction(Instruction::SetRepeat(self.repeat.clone()));
            }
            Message::TogglePause => {
                self.paused = !self.paused;
                return Action::instruction(Instruction::SetPause(self.paused));
            }
            Message::SeekForward(dur) => return Action::instruction(Instruction::SeekForward(dur)),
            Message::SeekBackward(dur) => {
                return Action::instruction(Instruction::SeekBackward(dur));
            }

            Message::SetShuffle(shuffle) => self.shuffle = shuffle,
            Message::SetRepeat(repeat) => self.repeat = repeat,
            Message::SetPause(paused) => self.paused = paused,

            Message::Resize(area) => self.resize(area),
        }

        Action::none()
    }

    #[allow(clippy::single_match)]
    fn handle_mouse(&self, ev: MouseEvent) -> Option<Message> {
        let x = ev.column;

        match ev.kind {
            MouseEventKind::Down(_button) => {
                if self.repeat_pos.contains(&x) {
                    return Some(Message::ToggleRepeat);
                }

                if self.seek_backward_pos.contains(&x) {
                    return Some(Message::SeekBackward(Duration::from_secs(5)));
                }

                if self.shuffle_pos.contains(&x) {
                    return Some(Message::ToggleShuffle);
                }

                if self.seek_forward_pos.contains(&x) {
                    return Some(Message::SeekForward(Duration::from_secs(5)));
                }

                if self.pause_pos.contains(&x) {
                    return Some(Message::TogglePause);
                }
            }
            _ => {}
        }

        None
    }

    fn view(&self, buf: &mut Buffer) {
        let area = self.area;
        debug_assert_eq!(area.height, Self::HEIGHT);

        let block = Block::new().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT);
        let inner_area = block.inner(area);

        block.render(area, buf);

        let [name_area, progress_area, button_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        )
        .areas(inner_area);

        {
            Line::raw(&self.name).centered().render(name_area, buf);
        }

        let [_, progress_area, _] = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Fill(1),
                Constraint::Fill(8),
                Constraint::Fill(1),
            ],
        )
        .areas(progress_area);

        {
            let width = progress_area.width;
            let progress = self.progress * 100.0;

            let one_cell_rat = 100.0 / width as f32;

            let till = (progress / one_cell_rat).round() as u16;
            for x in 0..width {
                let cell = buf
                    .cell_mut((progress_area.x + x, progress_area.y))
                    .unwrap();
                if x < till {
                    cell.set_char('#');
                } else {
                    cell.set_char('_');
                }
            }
        };

        {
            let [_, button_area, _] = Layout::new(
                Direction::Horizontal,
                [
                    Constraint::Fill(1),
                    Constraint::Length(19),
                    Constraint::Fill(1),
                ],
            )
            .areas(button_area);

            let control = match self.repeat {
                Repeat::None | Repeat::Queue => "[s] [<] [\u{23F8}] [>] [r]",
                Repeat::One => "[s] [<] [\u{23F8}] [>] [R]",
            };

            if self.paused {
                for i in self.pause_pos.clone() {
                    buf.cell_mut((i, button_area.y))
                        .unwrap()
                        .set_fg(ratatui::style::Color::Green);
                }
            } else {
                for i in self.pause_pos.clone() {
                    buf.cell_mut((i, button_area.y))
                        .unwrap()
                        .set_fg(ratatui::style::Color::Reset);
                }
            }

            match self.shuffle {
                Shuffle::None => {
                    for i in self.shuffle_pos.clone() {
                        buf.cell_mut((i, button_area.y))
                            .unwrap()
                            .set_fg(ratatui::style::Color::Reset);
                    }
                }
                Shuffle::Random => {
                    for i in self.shuffle_pos.clone() {
                        buf.cell_mut((i, button_area.y))
                            .unwrap()
                            .set_fg(ratatui::style::Color::Green);
                    }
                }
            };

            match self.repeat {
                Repeat::None => {
                    for i in self.repeat_pos.clone() {
                        buf.cell_mut((i, button_area.y))
                            .unwrap()
                            .set_fg(ratatui::style::Color::Reset);
                    }
                }
                Repeat::Queue | Repeat::One => {
                    for i in self.repeat_pos.clone() {
                        buf.cell_mut((i, button_area.y))
                            .unwrap()
                            .set_fg(ratatui::style::Color::Green);
                    }
                }
            }

            Line::raw(control).render(button_area, buf);
        }
    }
}
