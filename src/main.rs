use crate::{app::App, widget::Player};

mod app;
mod input;
mod music;
mod widget;

fn main() {
    let terminal = ratatui::init();

    // TODO: Parse config for dir in here
    // else use $HOME/music dir
    let player = Player::new("/home/lf/music");

    let app = App::with_player(player);
    app.start(terminal);

    ratatui::restore();
}
