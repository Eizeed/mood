use crossbeam_channel::Sender;
use ratatui::crossterm::event::{self, Event};

pub fn spawn_input(tx: Sender<Event>) {
    std::thread::spawn(move || {
        loop {
            let event = event::read().unwrap();
            let Ok(_) = tx.send(event) else { return };
        }
    });
}
