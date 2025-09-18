use std::time::Duration;

use crate::event::{Event, EventHandler};

mod config;
mod event;
mod app;
mod utils;
mod components;

fn main() -> color_eyre::Result<()> {
    let event_handler = EventHandler::new(Duration::from_millis(250));

    loop {
        //render

        match event_handler.next()? {
            Event::Input(key) => {}
            Event::Tick => (),
        }
    }
}
