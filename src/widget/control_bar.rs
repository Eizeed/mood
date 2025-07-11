use std::ops::Range;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::Widget,
};

#[derive(Debug)]
pub struct ControlBar {
    pub name: String,
    // duration
    // pos
    pub repeat: Repeat,
    pub random: bool,
    pub progress: Option<f32>,

    control_bar_y: Option<u16>,

    shuffle_pos: Option<Range<u16>>,
    seek_backward_pos: Option<Range<u16>>,
    pause_pos: Option<Range<u16>>,
    seek_forward_pos: Option<Range<u16>>,
    repeat_pos: Option<Range<u16>>,
}

#[derive(Debug)]
pub enum Repeat {
    None,
    RepeatQueue,
    RepeatOne,
}

impl ControlBar {
    pub fn new() -> Self {
        ControlBar {
            name: "".to_string(),
            repeat: Repeat::None,
            random: false,
            progress: None,

            control_bar_y: None,

            shuffle_pos: None,
            seek_backward_pos: None,
            pause_pos: None,
            seek_forward_pos: None,
            repeat_pos: None,
        }
    }

    pub fn control_bar_y(&self) -> u16 {
        self.control_bar_y.unwrap()
    }

    pub fn shuffle_pos(&self) -> &Range<u16> {
        self.shuffle_pos.as_ref().unwrap()
    }

    pub fn seek_backward_pos(&self) -> &Range<u16> {
        self.seek_backward_pos.as_ref().unwrap()
    }

    pub fn pause_pos(&self) -> &Range<u16> {
        self.pause_pos.as_ref().unwrap()
    }

    pub fn seek_forward_pos(&self) -> &Range<u16> {
        self.seek_forward_pos.as_ref().unwrap()
    }

    pub fn repeat_pos(&self) -> &Range<u16> {
        self.repeat_pos.as_ref().unwrap()
    }

    pub fn toggle_repeat(&mut self) {
        match self.repeat {
            Repeat::None => {
                self.repeat = Repeat::RepeatQueue;
            }
            Repeat::RepeatQueue => {
                self.repeat = Repeat::RepeatOne;
            }
            Repeat::RepeatOne => {
                self.repeat = Repeat::None;
            }
        }
    }
}

impl Widget for &mut ControlBar {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        debug_assert!(area.height == 3);

        let [name_area, progress_area, button_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        )
        .areas(area);

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
            let progress = match self.progress {
                Some(progress) => progress * 100.0,
                None => 0.0,
            };

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

            // This section looks really really REALLY bad. But it works!
            let control = match self.repeat {
                Repeat::None | Repeat::RepeatQueue => "[s] [<] [\u{23F8}] [>] [r]",
                Repeat::RepeatOne => "[s] [<] [\u{23F8}] [>] [R]",
            };

            let start = button_area.x;

            self.control_bar_y = Some(button_area.y);
            self.shuffle_pos = Some(start..start + 3);
            self.seek_backward_pos = Some(start + 4..start + 7);
            self.pause_pos = Some(start + 8..start + 11);
            self.seek_forward_pos = Some(start + 12..start + 15);
            self.repeat_pos = Some(start + 16..start + 19);

            if self.random {
                for i in self.shuffle_pos.as_ref().unwrap().clone() {
                    buf.cell_mut((i, button_area.y))
                        .unwrap()
                        .set_fg(ratatui::style::Color::Green);
                }
            } else {
                for i in self.shuffle_pos.as_ref().unwrap().clone() {
                    buf.cell_mut((i, button_area.y))
                        .unwrap()
                        .set_fg(ratatui::style::Color::Reset);
                }
            }

            match self.repeat {
                Repeat::None => {
                    for i in self.repeat_pos().clone() {
                        buf.cell_mut((i, button_area.y))
                            .unwrap()
                            .set_fg(ratatui::style::Color::Reset);
                    }
                }
                Repeat::RepeatQueue | Repeat::RepeatOne => {
                    for i in self.repeat_pos().clone() {
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
