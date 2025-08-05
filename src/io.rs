use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::{
    app::{Repeat, Shuffle},
    config::Config,
};
use lofty::{
    config::WriteOptions,
    file::{AudioFile, TaggedFileExt},
    read_from_path,
    tag::{ItemKey, ItemValue, Tag, TagItem},
};
use uuid::Uuid;

use crate::model::track::Track;

pub fn get_files<T: AsRef<Path>>(path: T, extension: &str) -> Vec<PathBuf> {
    let root = path.as_ref().to_path_buf();

    let mut files = vec![];
    let mut stack = vec![root];

    while let Some(dir) = stack.pop() {
        for entry in dir.read_dir().unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if let Some(ext) = path.extension() {
                if ext == extension {
                    files.push(path);
                }
            }
        }
    }

    files
}

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
                config.audio_dir_path = val.into();
            }
            "volume" => {
                if let Ok(vol) = val.parse::<f32>() {
                    let vol = vol.clamp(0.0, 1.0);
                    config.volume = vol;
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

pub fn save_config(config: Config) {
    let mut conf_dir = dirs::config_dir().expect("No config dir? :(");

    conf_dir.push("mood");

    std::fs::create_dir_all(&conf_dir).unwrap();

    conf_dir.push("mood.conf");

    let mut content = String::new();
    std::fs::OpenOptions::new()
        .read(true)
        .open(&conf_dir)
        .unwrap()
        .read_to_string(&mut content)
        .unwrap();

    let mut config_str = format!(
        "volume = {}\nshuffle = {}\nrepeat = {}\n",
        config.volume, config.shuffle as u8, config.repeat as u8
    );

    for line in content.lines() {
        if line.trim().starts_with("audio_path") {
            config_str.push_str(&format!("{line}\n"));
        }
    }

    let mut conf_file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(conf_dir)
        .unwrap();

    conf_file.write_all(config_str.as_bytes()).unwrap();
}

// Maybe this is bad, idk...
pub fn add_metadata<T>(paths: Vec<T>) -> Vec<Track>
where
    T: Into<PathBuf>,
{
    paths
        .into_iter()
        .map(|p| {
            let p = p.into();

            let mut tagged = read_from_path(&p).unwrap();
            let duration = tagged.properties().duration();

            let tag = match tagged.primary_tag_mut() {
                Some(tag) => {
                    if let Some(ItemValue::Text(uuid)) = tag
                        .get(&ItemKey::Unknown("MOOD_UUID".to_string()))
                        .map(|t| t.value())
                    {
                        if let Ok(uuid) = Uuid::parse_str(uuid) {
                            return Track {
                                uuid,
                                duration,
                                path: p,
                            };
                        };
                    }

                    tag
                }
                None => {
                    let tag = Tag::new(tagged.file_type().primary_tag_type());
                    tagged.insert_tag(tag);
                    tagged.primary_tag_mut().unwrap()
                }
            };

            let uuid = Uuid::new_v4();

            tag.insert_unchecked(TagItem::new(
                ItemKey::Unknown("MOOD_UUID".into()),
                ItemValue::Text(uuid.to_string()),
            ));

            tagged.save_to_path(&p, WriteOptions::default()).unwrap();
            Track {
                uuid,
                duration,
                path: p,
            }
        })
        .collect()
}
