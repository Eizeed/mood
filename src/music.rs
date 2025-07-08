use std::{
    io::BufReader,
    path::PathBuf,
};

use crossbeam_channel::{Receiver, Sender};
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

pub enum Message {
    TrackEnded,
}

struct CustomSource<T>
where
    T: Source,
    T::Item: rodio::Sample,
{
    inner: T,
    main_handle: Sender<Message>,
}

impl<T> Iterator for CustomSource<T>
where
    T: Source,
    T::Item: rodio::Sample,
{
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next();

        if let None = next {
            self.main_handle.send(Message::TrackEnded).unwrap();
        }

        next
    }
}

impl<T> Source for CustomSource<T>
where
    T: Source,
    T::Item: rodio::Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.total_duration()
    }
}

pub fn spawn_music(rx: Receiver<Command>, tx: Sender<Message>) {
    std::thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        loop {
            // TODO: Need to create custom Source with callbacks
            let Ok(command) = rx.recv() else { return };
            match command {
                Command::Play(path) => {
                    let file = std::fs::File::open(path).unwrap();
                    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                    let source = CustomSource {
                        inner: source,
                        main_handle: tx.clone(),
                    };

                    // let _dur = source.total_duration();

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
