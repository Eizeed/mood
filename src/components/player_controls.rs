use std::cell::Cell;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Block, Widget, WidgetRef},
};

pub struct PlayerControlsComponent {
    pub progress: Cell<u16>,
}

impl PlayerControlsComponent {
    pub fn new() -> Self {
        PlayerControlsComponent {
            progress: Cell::new(0),
        }
    }
}

impl WidgetRef for PlayerControlsComponent {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let area = {
            let border = Block::bordered();
            let a = border.inner(area);
            border.render(area, buf);
            a
        };

        let [name_area, progress_area, control_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        )
        .areas(area);

        let [_, progress_area, _] = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Fill(1),
                Constraint::Fill(8),
                Constraint::Fill(1),
            ],
        )
        .areas(progress_area);

        Line::raw("Track name").centered().render(name_area, buf);

        let done = progress_area.width * self.progress.get() / 100;
        eprintln!("{}, {}", progress_area.width, self.progress.get());
        for i in 0..progress_area.width {
            buf.cell_mut((i + progress_area.x, progress_area.y))
                .map(|c| c.set_char(if i < done { '#' } else { '-' }));
        }

        Line::raw("Controls").centered().render(control_area, buf);
    }
}
