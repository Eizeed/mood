use std::path::PathBuf;

pub enum Command {
    SetCurrentTrack { path: PathBuf, }
}
