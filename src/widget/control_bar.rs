use std::ops::Range;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Block, Borders, StatefulWidget, Widget},
};

use crate::app::{Context, Repeat, Shuffle};

#[derive(Debug)]
pub struct ControlBar {
    pub name: String,
    // TODO: in future when i will bring more styles
    // Create a tracker for duration something like
    // 0:52---3:22
    // Or somthing simmilar

    // duration
    // pos

    pub control_bar_y: u16,

    pub shuffle_pos: Range<u16>,
    pub seek_backward_pos: Range<u16>,
    pub pause_pos: Range<u16>,
    pub seek_forward_pos: Range<u16>,
    pub repeat_pos: Range<u16>,
}

impl ControlBar {
    pub const HEIGHT: u16 = 4;

    pub fn new(area: Rect) -> Self {
        let [_, _, _, button_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        )
        .areas(area);

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

            control_bar_y,

            shuffle_pos,
            seek_backward_pos,
            pause_pos,
            seek_forward_pos,
            repeat_pos,
        }
    }

    pub fn resize(&mut self, area: Rect) {
        let [_, _, _, button_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        )
        .areas(area);

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
    }
}

impl StatefulWidget for &ControlBar {
    type State = Context;

    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        debug_assert!(area.height == 4);

        let [border, name_area, progress_area, button_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        )
        .areas(area);

        Block::new().borders(Borders::TOP).render(border, buf);

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
            let progress =  state.progress * 100.0;

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

            let control = match state.repeat {
                Repeat::None | Repeat::Queue => "[s] [<] [\u{23F8}] [>] [r]",
                Repeat::One => "[s] [<] [\u{23F8}] [>] [R]",
            };

            match state.shuffle {
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

            match state.repeat {
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
