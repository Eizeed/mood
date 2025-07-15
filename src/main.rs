use ratatui::layout::Rect;

use crate::{app::App, io::get_config};

mod app;
mod config;
mod input;
mod io;
mod music;
mod widget;

fn main() {
    let terminal = ratatui::init();

    let config = get_config();

    let size = terminal.size().unwrap();
    let area = Rect::new(0, 0, size.width, size.height);

    let app = App::new(config, area);
    app.start(terminal);

    ratatui::restore();
}
