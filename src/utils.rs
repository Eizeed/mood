use crate::event::{Command, Event};

pub fn spawn_audio(
    event_tx: crossbeam_channel::Sender<Event>,
    command_rx: crossbeam_channel::Receiver<Command>,
) -> color_eyre::Result<()> {
    std::thread::spawn(move || {});
    Ok(())
}
