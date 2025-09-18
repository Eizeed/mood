use std::{path::PathBuf, time::Duration};

use rusqlite::Connection;

use crate::{
    app::App,
    config::Config,
    event::Event,
    utils::{spawn_audio_thread, spawn_event_emmiter},
};

mod app;
mod components;
mod config;
mod event;
mod utils;

fn main() -> color_eyre::Result<()> {
    let tickrate = Duration::from_millis(250);

    let (event_tx, event_rx) = crossbeam_channel::unbounded();
    let (command_tx, command_rx) = crossbeam_channel::unbounded();

    spawn_event_emmiter(event_tx.clone(), tickrate)?;
    spawn_audio_thread(command_rx, event_tx.clone())?;

    let config = Config::new(PathBuf::from("/home/lf/music"));

    let sqlite = Connection::open("db.db3")?;

    let app = App::new(command_tx, config, sqlite);

    loop {
        //render

        match event_rx.recv()? {
            Event::Input(key) => {}
            Event::Tick => (),
            Event::Audio(audio) => {}
        }
    }
}
