use ratatui::layout::Rect;

use crate::app::Player;

mod app;
mod input;
mod io;
mod music;
mod widget;
mod model;
mod action;
mod task;
mod config;

fn main() {
    let terminal = ratatui::init();

    let size = terminal.size().unwrap();
    let area = Rect::new(0, 0, size.width, size.height);

    let app = Player::new(area);
    app.start(terminal);

    ratatui::restore();
}
