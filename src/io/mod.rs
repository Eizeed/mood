use std::path::{Path, PathBuf};

use color_eyre::Result;
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::read_from_path;
use lofty::tag::{ItemKey, ItemValue, Tag, TagItem};
use uuid::Uuid;

use crate::models::Track;

pub fn get_files(root: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let root = root.to_path_buf();

    let mut files = vec![];
    let mut stack = vec![root];

    while let Some(dir) = stack.pop() {
        for entry in dir.read_dir()? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Some(ext) = path.extension() {
                if ext == extension {
                    files.push(path);
                }
            }
        }
    }

    Ok(files)
}

pub fn add_metadata<T>(paths: Vec<T>) -> Vec<Track>
where
    T: Into<PathBuf>,
{
    paths
        .into_iter()
        .map(|p| {
            let path = p.into();

            let mut tagged = read_from_path(&path).unwrap();
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
                                path,
                            };
                        };
                    }

                    tag
                }
                None => {
                    let tag =
                        Tag::new(tagged.file_type().primary_tag_type());
                    tagged.insert_tag(tag);
                    tagged.primary_tag_mut().unwrap()
                }
            };

            let uuid = Uuid::new_v4();

            tag.insert_unchecked(TagItem::new(
                ItemKey::Unknown("MOOD_UUID".into()),
                ItemValue::Text(uuid.to_string()),
            ));

            tagged.save_to_path(&path, WriteOptions::default()).unwrap();
            Track {
                uuid,
                duration,
                path,
            }
        })
        .collect()
}
