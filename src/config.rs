use std::path::PathBuf;

use crate::app::{Repeat, Shuffle};

#[derive(Debug)]
pub struct Config {
    pub audio_dir: PathBuf,
    pub volume: f32,
    pub shuffle: Shuffle,
    pub repeat: Repeat,
}

impl Default for Config {
    fn default() -> Self {
        let mut home = dirs::home_dir().expect("No home dir? :(");
        home.push("music");

        Config {
            audio_dir: home,
            volume: 1.0,
            shuffle: Shuffle::None,
            repeat: Repeat::None,
        }
    }
}
