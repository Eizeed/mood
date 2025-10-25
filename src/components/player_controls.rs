use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget, WidgetRef};

pub struct PlayerControlsComponent {
    pub name: Option<String>,
    pub progress: u16,
}

impl PlayerControlsComponent {
    pub fn new() -> Self {
        PlayerControlsComponent {
            name: None,
            progress: 0,
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

        Line::raw(self.name.as_ref().map(|s| s.as_str()).unwrap_or("No name"))
            .centered()
            .render(name_area, buf);

        let done = progress_area.width * self.progress / 100;
        eprintln!("{}, {}", progress_area.width, self.progress);
        for i in 0..progress_area.width {
            buf.cell_mut((i + progress_area.x, progress_area.y))
                .map(|c| c.set_char(if i < done { '#' } else { '-' }));
        }

        Line::raw("Controls").centered().render(control_area, buf);
    }
}
