use crossterm::event;
use ratatui::widgets::Widget;

use crate::{app::App, player::Player};

mod app;
mod cursor;
mod player;
mod music;

fn main() {
    let mut terminal = ratatui::init();

    let player = Player::new("/home/lf/music");
    let mut app = App::with_player(player);

    loop {
        terminal
            .draw(|f| {
                app.render(f.area(), f.buffer_mut());
            })
            .expect("failed to draw frame");

        app.handle_event(event::read().unwrap());

        if app.should_exit {
            break;
        }
    }
    ratatui::restore();
}
