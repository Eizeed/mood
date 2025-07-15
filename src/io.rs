use std::path::{Path, PathBuf};

use crate::{
    app::{Repeat, Shuffle},
    config::Config,
};

pub fn get_config() -> Config {
    let mut config = Config::default();

    let mut conf_dir = dirs::config_dir().expect("No config dir? :(");

    conf_dir.push("mood");

    std::fs::create_dir_all(&conf_dir).unwrap();

    conf_dir.push("mood.conf");
    let Some(blob) = std::fs::read_to_string(conf_dir).ok() else {
        return config;
    };

    for line in blob.lines() {
        let mut iter = line.split('=');
        let key = iter.next().unwrap().trim().to_string();
        let val = iter.next().unwrap().trim().to_string();

        match key.as_str() {
            "audio_path" => {
                config.audio_dir = val.into();
            }
            "volume" => {
                let vol = val.parse();
                match vol {
                    Ok(vol) => {
                        let vol = if vol > 1.0 {
                            1.0
                        } else if vol < 0.0 {
                            0.0
                        } else {
                            vol
                        };

                        config.volume = vol;
                    }
                    Err(_) => {}
                }
            }
            "shuffle" => {
                config.shuffle = match val.as_str() {
                    "0" => Shuffle::None,
                    "1" => Shuffle::Random,
                    _ => config.shuffle,
                }
            }
            "repeat" => {
                config.repeat = match val.as_str() {
                    "0" => Repeat::None,
                    "1" => Repeat::Queue,
                    "2" => Repeat::One,
                    _ => config.repeat,
                }
            }
            _ => {}
        };
    }

    config
}

pub fn get_files<T: AsRef<Path>>(path: T, extension: &str) -> Vec<PathBuf> {
    let root = path.as_ref().to_path_buf();

    let mut files = vec![];
    let mut stack = vec![root];

    while let Some(dir) = stack.pop() {
        for entry in dir.read_dir().unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.extension() {
                    Some(ext) => {
                        if ext == extension {
                            files.push(path);
                        }
                    }
                    None => {}
                }
            }
        }
    }

    files
}
