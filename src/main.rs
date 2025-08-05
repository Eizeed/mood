use crossterm::{ExecutableCommand, event::DisableMouseCapture};
use ratatui::layout::Rect;
use std::io::stdout;

use crate::app::Player;

mod action;
mod app;
mod config;
mod input;
mod io;
mod model;
mod music;
mod task;
mod widget;

fn main() {
    set_panic_hook();
    let terminal = ratatui::init();

    let size = terminal.size().unwrap();
    let area = Rect::new(0, 0, size.width, size.height);

    let app = Player::new(area);
    app.start(terminal);

    ratatui::restore();
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        stdout().execute(DisableMouseCapture).unwrap();

        hook(info);
    }));
}
