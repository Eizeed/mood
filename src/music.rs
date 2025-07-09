use std::{fmt::Debug, fs::File, io::BufReader, time::Duration};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use rodio::{Decoder, OutputStream, Sink, Source};

pub enum Command {
    Play(Decoder<BufReader<File>>),
    Pause,
    Resume,

    VolumeUp(f32),
    VolumeDown(f32),

    SeekForward(Duration),
    SeekBackward(Duration),
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Play(_) => f.write_str("Play"),
            Command::Pause => f.write_str("Pause"),
            Command::Resume => f.write_str("Resume"),
            Command::VolumeUp(_) => f.write_str("VolumeUp"),
            Command::VolumeDown(_) => f.write_str("VolumeDown"),
            Command::SeekForward(_) => f.write_str("SeekForward"),
            Command::SeekBackward(_) => f.write_str("SeekBackward"),
        }
    }
}

impl Command {
    pub fn play<T>(source: T) -> Self
    where
        T: Into<Decoder<BufReader<File>>>,
    {
        Command::Play(source.into())
    }

    pub fn resume() -> Self {
        Command::Resume
    }
    pub fn pause() -> Self {
        Command::Pause
    }

    pub fn volume_up<T>(vol: T) -> Self
    where
        T: Into<f32>,
    {
        Self::VolumeUp(vol.into())
    }

    pub fn volume_down<T>(vol: T) -> Self
    where
        T: Into<f32>,
    {
        Self::VolumeDown(vol.into())
    }

    pub fn seek_forward<T>(dur: T) -> Self
    where
        T: Into<Duration>,
    {
        Self::SeekForward(dur.into())
    }

    pub fn seek_backward<T>(dur: T) -> Self
    where
        T: Into<Duration>,
    {
        Self::SeekBackward(dur.into())
    }
}

#[derive(Debug)]
pub enum Message {
    TrackEnded(Method),
    CurrentVolume(f32),
    CurrentPos(Duration),
}

#[derive(Debug)]
pub enum Method {
    Normal,
    Seek,
}

struct NotifySource<T>
where
    T: Source,
    T::Item: rodio::Sample,
{
    inner: T,
    main_handle: Sender<Message>,
}

impl<T> Iterator for NotifySource<T>
where
    T: Source,
    T::Item: rodio::Sample,
{
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next();

        if let None = next {
            self.main_handle
                .send(Message::TrackEnded(Method::Normal))
                .unwrap();
        }

        next
    }
}

impl<T> Source for NotifySource<T>
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
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.inner.try_seek(pos)
    }
}

pub fn spawn_music(rx: Receiver<Command>, tx: Sender<Message>) {
    std::thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        tx.send(Message::CurrentVolume(sink.volume())).unwrap();

        loop {
            if sink.empty() {
                sink.stop();
            }

            let command = match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(command) => command,
                Err(err) => match err {
                    RecvTimeoutError::Timeout => {
                        if !sink.empty() && !sink.is_paused() {
                            tx.send(Message::CurrentPos(sink.get_pos())).unwrap();
                        }
                        continue;
                    }
                    RecvTimeoutError::Disconnected => return,
                },
            };

            match command {
                Command::Play(source) => {
                    let source = NotifySource {
                        inner: source,
                        main_handle: tx.clone(),
                    };

                    sink.clear();
                    sink.append(source);
                    sink.play();

                    // Min is 27ms.
                    // This one is driving me crazy.
                    // Sink.position is mutex. It has some delay
                    // on apped function. It locks control and copies it
                    // in new source and so if i call get_pos() instantly
                    // it will give me old pos...
                    //
                    // TODO: Oneday get rid of this
                    // Write a fork or use something like
                    // Symphonia and cpal
                    std::thread::sleep(Duration::from_millis(50));

                    tx.send(Message::CurrentPos(sink.get_pos())).unwrap();
                }
                Command::Pause => {
                    if !sink.empty() {
                        sink.pause();
                    }
                }
                Command::Resume => {
                    if !sink.empty() {
                        sink.play();
                        tx.send(Message::CurrentPos(sink.get_pos())).unwrap();
                    }
                }
                Command::VolumeUp(vol) => {
                    if sink.volume() + vol >= 1.0 {
                        sink.set_volume(1.0);
                    } else {
                        sink.set_volume(sink.volume() + vol);
                    }

                    tx.send(Message::CurrentVolume(sink.volume())).unwrap();
                }
                Command::VolumeDown(vol) => {
                    if sink.volume() - vol <= 0.0 {
                        sink.set_volume(0.0);
                    } else {
                        sink.set_volume(sink.volume() - vol);
                    }

                    tx.send(Message::CurrentVolume(sink.volume())).unwrap();
                }
                Command::SeekForward(duration) => {
                    let pos = sink.get_pos();

                    if !sink.empty() {
                        // if pos <= max_duration {
                        sink.try_seek(pos + duration).unwrap();

                        tx.send(Message::CurrentPos(sink.get_pos())).unwrap();

                        if sink.empty() {
                            tx.send(Message::TrackEnded(Method::Seek)).unwrap();
                            sink.clear();
                        }
                        // }
                    }
                }
                Command::SeekBackward(duration) => {
                    if !sink.empty() {
                        let pos = sink.get_pos();
                        let new_pos = pos.saturating_sub(duration);

                        sink.try_seek(new_pos).unwrap();

                        tx.send(Message::CurrentPos(sink.get_pos())).unwrap();
                    }
                }
            }
        }
    });
}
