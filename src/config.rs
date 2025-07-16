use std::path::PathBuf;

use crate::app::{Repeat, Shuffle};

#[derive(Debug)]
pub struct Config {
    pub audio_dir_path: PathBuf,
    pub database_path: PathBuf,
    pub volume: f32,
    pub shuffle: Shuffle,
    pub repeat: Repeat,
}

impl Default for Config {
    fn default() -> Self {
        let mut home = dirs::home_dir().expect("No home dir? :(");
        home.push("music");

        let mut data = dirs::data_dir().expect("No data dir? :(");
        data.push("mood");
        if !data.exists() {
            std::fs::create_dir_all(&data).unwrap();
        }

        data.push("sqlite.db");

        Config {
            audio_dir_path: home,
            database_path: data,
            volume: 1.0,
            shuffle: Shuffle::None,
            repeat: Repeat::None,
        }
    }
}
