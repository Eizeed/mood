use std::path::{Path, PathBuf};

use crate::config::Config;

pub fn get_config() -> Config {
    let mut conf_dir = dirs::config_dir().expect("No config dir? :(");
    let mut home_dir = dirs::home_dir().expect("No home dir? :(");
    home_dir.push("music");

    conf_dir.push("mood");

    std::fs::create_dir_all(&conf_dir).unwrap();

    conf_dir.push("mood.conf");
    let Some(blob) = std::fs::read_to_string(conf_dir).ok() else {
        return Config {
            audio_dir: home_dir,
        };
    };

    for line in blob.lines() {
        let mut iter = line.split('=');
        let key = iter.next().unwrap().trim().to_string();
        let val = iter.next().unwrap().trim().to_string();

        if key == "audio_path" {
            return Config {
                audio_dir: val.into(),
            };
        }
    }

    Config {
        audio_dir: home_dir,
    }
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
