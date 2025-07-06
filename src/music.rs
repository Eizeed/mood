use std::{io::BufReader, path::PathBuf, sync::mpsc::Receiver};

use rodio::{OutputStream, Sink, Source};

pub enum Command {
    Play(PathBuf),
    Pause,
    Resume,
}

impl Command {
    pub fn play<T>(path: T) -> Self
    where
        T: Into<PathBuf>,
    {
        Command::Play(path.into())
    }

    pub fn resume() -> Self {
        Command::Resume
    }

    pub fn pause() -> Self {
        Command::Pause
    }
}

pub fn spawn_music(rx: Receiver<Command>) {
    std::thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        loop {
            let Ok(command) = rx.recv() else { return };
            match command {
                Command::Play(path) => {
                    let file = std::fs::File::open(path).unwrap();
                    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                    let _dur = source.total_duration();

                    sink.stop();
                    sink.play();
                    sink.append(source);
                }
                Command::Pause => {
                    sink.pause();
                }
                Command::Resume => {
                    sink.play();
                }
            }
        }
    });
}
