use crossbeam_channel::Sender;
use ratatui::crossterm::event::{self, Event};

// Create a channel to get status of track like
// Seeking - To prevent from weird noises during search
//           Need to pause track and skip until user stops
//           seeking. If this state is used,
//           use event::poll() with duration like 100ms
//
// None    - Default behaviour with blocking call
//
// Maybe try to find some other way to do that
pub fn spawn_input(tx: Sender<Event>) {
    std::thread::spawn(move || {
        loop {
            let event = event::read().unwrap();
            let Ok(_) = tx.send(event) else { return };
        }
    });
}
