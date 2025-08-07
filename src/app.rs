use std::{
    collections::VecDeque,
    io::{BufReader, stdout},
    rc::Rc,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender};
use ratatui::{
    Terminal,
    buffer::Buffer,
    crossterm::{
        ExecutableCommand,
        event::{
            DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
        },
    },
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Backend,
    widgets::Widget,
};
use rusqlite::Connection;

use crate::{
    component::{
        Component,
        control_bar::{self, ControlBar},
        fallback::Fallback,
        header::Header,
        playlist::{self, Playlist},
        tracklist::{self, Tracklist},
    },
    config::Config,
    input::spawn_input,
    io::{add_metadata, get_config, get_files, save_config},
    model::{self, PlaylistMd, track::Track},
    music::{self, Command, spawn_music},
    task::Task,
};

pub struct Player {
    header: Header,
    tracklist: Tracklist,
    playlist: Playlist,
    control_bar: ControlBar,

    focused_widget: Focus,
    mode: Mode,

    paused: bool,
    volume: f32,
    repeat: Repeat,
    shuffle: Shuffle,
    should_exit: bool,

    last_seek_timer: Instant,

    fallback_render: bool,

    audio_tx: Sender<Command>,
    audio_rx: Receiver<music::Message>,
    input_rx: Receiver<Event>,

    db_conn: Connection,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Tracklist(tracklist::Instruction),
    Playlist(playlist::Instruction),
    ControlBar(control_bar::Instruction),
}

#[derive(Debug, Clone)]
pub enum Message {
    Tracklist(tracklist::Message),
    Playlist(playlist::Message),
    ControlBar(control_bar::Message),

    SeekForward(Duration),
    SeekBackward(Duration),

    VolumeUp(f32),
    VolumeDown(f32),

    ToggleShufle,
    ToggleRepeat,

    SetBaseQueue,
    Pause,
    SkipToNext,
    SkipToPrev,

    SetProgress(f32),
    SetVolume(f32),
    PlayNext,

    Resize(Rect),

    Exit,
}

#[derive(Debug, Clone)]
pub enum Shuffle {
    None,
    Random,
}

#[derive(Debug, Clone)]
pub enum Repeat {
    None,
    Queue,
    One,
}

#[derive(Debug, Clone)]
pub enum Focus {
    Tracklist,
    Playlist,
}

#[derive(Debug, Clone)]
pub enum Mode {
    Default,
    Write,
}

impl Player {
    pub fn new(area: Rect) -> Self {
        let config = get_config();
        let (main_audio_tx, main_audio_rx) = crossbeam_channel::bounded::<Command>(64);
        let (audio_main_tx, audio_main_rx) = crossbeam_channel::bounded::<music::Message>(64);

        let (input_main_tx, input_main_rx) = crossbeam_channel::bounded::<Event>(64);

        let conn = Connection::open(config.database_path).unwrap();

        spawn_music(main_audio_rx, audio_main_tx);
        spawn_input(input_main_tx);

        main_audio_tx
            .send(Command::SetVolume(config.volume))
            .unwrap();

        let paths = get_files(config.audio_dir_path, "mp3");

        let tracks: Rc<[Track]> = add_metadata(paths).into();

        let [header_area, playlist_area, control_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(4),
            ],
        )
        .areas(area);

        let shuffle = config.shuffle;
        let repeat = config.repeat;

        Player {
            header: Header::new("LOL NO PLAYLIST".to_string(), header_area),
            playlist: Playlist::new(&conn, playlist_area),
            tracklist: Tracklist::new(tracks, playlist_area, shuffle.clone(), repeat.clone()),
            control_bar: ControlBar::new(control_area, 0.0, shuffle.clone(), repeat.clone()),

            focused_widget: Focus::Tracklist,
            mode: Mode::Default,

            volume: config.volume,
            paused: false,
            shuffle,
            repeat,

            should_exit: false,

            // tracks: tracks.into(),
            last_seek_timer: Instant::now(),

            fallback_render: false,

            audio_tx: main_audio_tx,
            audio_rx: audio_main_rx,
            input_rx: input_main_rx,

            db_conn: conn,
        }
    }

    pub fn start<B: Backend>(mut self, mut terminal: Terminal<B>) {
        stdout().execute(EnableMouseCapture).unwrap();

        let mut task_queue = VecDeque::with_capacity(16);
        loop {
            terminal
                .draw(|f| self.render(f.area(), f.buffer_mut()))
                .expect("failed to draw frame");

            crossbeam_channel::select_biased! {
                recv(self.input_rx) -> event => {
                    let event = event.unwrap();
                    if let Some(curr_message) = self.handle_event(event) {
                        let task = self.update(curr_message);
                        self.handle_task(task, &mut task_queue);
                    }
                }
                recv(self.audio_rx) -> msg => {
                    let msg = msg.unwrap();

                    if let Some(curr_message) = self.handle_audio(msg) {
                        let task = self.update(curr_message);
                        self.handle_task(task, &mut task_queue);
                    }

                    for msg in self.audio_rx.clone().try_iter() {
                        if let Some(curr_message) = self.handle_audio(msg) {
                            let task = self.update(curr_message);
                            self.handle_task(task, &mut task_queue);
                        }
                    }
                }
            };

            if self.should_exit {
                break;
            }
        }

        save_config(Config {
            volume: self.volume,
            shuffle: self.shuffle,
            repeat: self.repeat,
            ..Default::default()
        });

        stdout().execute(DisableMouseCapture).unwrap();
    }

    fn handle_task(&mut self, task: Task<Message>, task_queue: &mut VecDeque<Task<Message>>) {
        task_queue.push_back(task);

        while let Some(task) = task_queue.pop_front() {
            let mut messages = task.into_inner();
            while let Some(message) = messages.pop() {
                let task = self.update(message);
                if !task.is_none() {
                    task_queue.push_front(task);
                }
            }
        }
    }

    fn handle_event(&self, ev: Event) -> Option<Message> {
        match ev {
            Event::Key(c) => {
                if c.kind != KeyEventKind::Press {
                    return None;
                }

                match self.mode {
                    Mode::Default => {
                        let msg = match c.code {
                            KeyCode::Esc => Some(Message::Exit),
                            KeyCode::Char('j') => match c.modifiers {
                                KeyModifiers::CONTROL => Some(Message::VolumeDown(0.05)),
                                _ => None,
                            },
                            KeyCode::Char('k') => match c.modifiers {
                                KeyModifiers::CONTROL => Some(Message::VolumeUp(0.05)),
                                _ => None,
                            },
                            KeyCode::Char('h') => match c.modifiers {
                                KeyModifiers::CONTROL => Some(Message::SkipToPrev),
                                KeyModifiers::NONE => {
                                    Some(Message::SeekBackward(Duration::from_secs(5)))
                                }
                                _ => None,
                            },
                            KeyCode::Char('l') => match c.modifiers {
                                KeyModifiers::CONTROL => Some(Message::SkipToNext),
                                KeyModifiers::NONE => {
                                    Some(Message::SeekForward(Duration::from_secs(5)))
                                }
                                _ => None,
                            },
                            KeyCode::Char('s') => match c.modifiers {
                                KeyModifiers::NONE => Some(Message::ToggleShufle),
                                _ => None,
                            },
                            KeyCode::Char('r') => match c.modifiers {
                                KeyModifiers::NONE => Some(Message::ToggleRepeat),
                                _ => None,
                            },
                            KeyCode::Char('e') => Some(Message::SetBaseQueue),
                            KeyCode::Char(' ') => Some(Message::Pause),
                            _ => None,
                        };

                        if msg.is_none() {
                            match &self.focused_widget {
                                Focus::Tracklist => self
                                    .tracklist
                                    .handle_input(c.code, c.modifiers)
                                    .map(Message::Tracklist),

                                Focus::Playlist => self
                                    .playlist
                                    .handle_input(c.code, c.modifiers)
                                    .map(Message::Playlist),
                            }
                        } else {
                            msg
                        }
                    }
                    Mode::Write => match self.focused_widget {
                        Focus::Tracklist => None,
                        Focus::Playlist => self
                            .playlist
                            .handle_input(c.code, c.modifiers)
                            .map(Message::Playlist),
                    },
                }
            }
            Event::Mouse(ev) => {
                let y = ev.row;

                if y == self.control_bar.control_bar_y {
                    self.control_bar.handle_mouse(ev).map(Message::ControlBar)
                } else {
                    None
                }
            }
            Event::Resize(cols, rows) => {
                let area = Rect::new(0, 0, cols, rows);
                Some(Message::Resize(area))
            }
            _ => None,
        }
    }

    fn handle_audio(&self, msg: music::Message) -> Option<Message> {
        match msg {
            music::Message::TrackEnded => Some(Message::PlayNext),
            music::Message::CurrentVolume(vol) => Some(Message::SetVolume(vol)),
            music::Message::CurrentPos(pos) => {
                let dur = self.tracklist.get_current().map(|c| c.duration);
                dur.map(|dur| {
                    Message::SetProgress((pos.as_millis() as f32) / (dur.as_millis() as f32))
                })
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SeekForward(dur) => self.seek_forward(dur),
            Message::SeekBackward(dur) => self.seek_backward(dur),
            Message::VolumeUp(vol) => {
                self.audio_tx.send(music::Command::volume_up(vol)).unwrap();
                return Task::none();
            }
            Message::VolumeDown(vol) => {
                self.audio_tx
                    .send(music::Command::volume_down(vol))
                    .unwrap();
                return Task::none();
            }
            Message::SetBaseQueue => {
                return Task::new(Message::Tracklist(tracklist::Message::SetBaseQueue));
            }
            Message::Pause => {
                self.paused = !self.paused;
                if self.paused {
                    self.audio_tx.send(Command::pause()).unwrap();
                } else {
                    self.audio_tx.send(Command::resume()).unwrap();
                }
                return Task::new(Message::ControlBar(control_bar::Message::SetPause(
                    self.paused,
                )));
            }
            Message::SkipToNext => {
                return Task::new(Message::Tracklist(tracklist::Message::SkipToNext));
            }
            Message::SkipToPrev => {
                return Task::new(Message::Tracklist(tracklist::Message::SkipToPrev));
            }

            Message::ToggleShufle => {
                match self.shuffle {
                    Shuffle::None => self.shuffle = Shuffle::Random,
                    Shuffle::Random => self.shuffle = Shuffle::None,
                }

                return Task::new(Message::ControlBar(control_bar::Message::SetShuffle(
                    self.shuffle.clone(),
                )));
            }
            Message::ToggleRepeat => {
                match self.repeat {
                    Repeat::None => self.repeat = Repeat::Queue,
                    Repeat::Queue => self.repeat = Repeat::One,
                    Repeat::One => self.repeat = Repeat::None,
                }

                return Task::new(Message::ControlBar(control_bar::Message::SetRepeat(
                    self.repeat.clone(),
                )));
            }
            Message::SetProgress(progress) => {
                let action = self
                    .control_bar
                    .update(control_bar::Message::SetProgress(progress))
                    .map(Message::ControlBar)
                    .map_instruction(Instruction::ControlBar);

                let instruction_message = self.perform(action.instruction);

                return Task::batch(vec![action.task, instruction_message]);
            }
            Message::SetVolume(vol) => self.volume = vol,
            Message::PlayNext => {
                let action = self
                    .tracklist
                    .update(tracklist::Message::GetNext)
                    .map(Message::Tracklist)
                    .map_instruction(Instruction::Tracklist);

                let instruction_message = self.perform(action.instruction);

                return Task::batch(vec![action.task, instruction_message]);
            }
            Message::Resize(area) => {
                if area.width < 21 || area.height < 16 {
                    self.fallback_render = true;
                } else {
                    self.fallback_render = false;
                    let [header_area, main_area, control_area] = Layout::new(
                        Direction::Vertical,
                        [
                            Constraint::Length(2),
                            Constraint::Fill(1),
                            Constraint::Length(4),
                        ],
                    )
                    .areas(area);

                    self.header.resize(header_area);

                    // Handle resizing messages later
                    self.tracklist.update(tracklist::Message::Resize(main_area));
                    self.playlist.update(playlist::Message::Resize(main_area));
                    self.control_bar
                        .update(control_bar::Message::Resize(control_area));
                }
            }
            Message::Exit => self.should_exit = true,

            Message::Tracklist(message) => {
                let action = self
                    .tracklist
                    .update(message)
                    .map(Message::Tracklist)
                    .map_instruction(Instruction::Tracklist);

                let instruction_message = self.perform(action.instruction);

                return Task::batch(vec![instruction_message, action.task]);
            }
            Message::Playlist(message) => {
                let action = self
                    .playlist
                    .update(message)
                    .map(Message::Playlist)
                    .map_instruction(Instruction::Playlist);

                let instruction_message = self.perform(action.instruction);

                return Task::batch(vec![instruction_message, action.task]);
            }
            Message::ControlBar(message) => {
                let action = self
                    .control_bar
                    .update(message)
                    .map(Message::ControlBar)
                    .map_instruction(Instruction::ControlBar);

                let instruction_message = self.perform(action.instruction);

                return Task::batch(vec![action.task, instruction_message]);
            }
        }

        Task::none()
    }

    fn perform(&mut self, instruction: Option<Instruction>) -> Task<Message> {
        let Some(instruction) = instruction else {
            return Task::none();
        };

        match instruction {
            Instruction::Tracklist(instruction) => {
                use tracklist::Instruction;

                match instruction {
                    Instruction::Play(track) => {
                        let file = std::fs::File::open(&track.path).unwrap();
                        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

                        self.audio_tx.send(Command::play(source)).unwrap();
                        return Task::new(Message::ControlBar(control_bar::Message::SetName(
                            track
                                .path
                                .file_stem()
                                .unwrap()
                                .to_string_lossy()
                                .to_string(),
                        )));
                    }
                    Instruction::FocusPlaylist => {
                        self.focused_widget = Focus::Playlist;
                    }
                    Instruction::AddToPlaylist(track) => {
                        self.focused_widget = Focus::Playlist;
                        self.playlist.update(playlist::Message::SetTrack(track));
                    }
                    Instruction::RemoveFromPlaylist(playlist, track) => {
                        let playlist = playlist.remove_track(track, &self.db_conn);
                        return Task::new(Message::Tracklist(tracklist::Message::UpdatePlaylist(
                            playlist,
                        )));
                    }
                    Instruction::SetHeader(header) => {
                        self.header.playlist_name = header;
                    }
                }
            }
            Instruction::Playlist(instruction) => {
                use playlist::Instruction;

                match instruction {
                    Instruction::FocusTracklist => self.focused_widget = Focus::Tracklist,
                    Instruction::SetPlaylist(md) => {
                        let playlist = model::Playlist::from_playlistmd(md, &self.db_conn);
                        self.focused_widget = Focus::Tracklist;
                        self.header.playlist_name = playlist.name.clone();
                        return Task::new(Message::Tracklist(tracklist::Message::SetPlaylist(
                            playlist,
                        )));
                    }
                    Instruction::AddTrackToPlaylist(playlist_md, track) => {
                        playlist_md.insert_track(track, &self.db_conn);
                        self.focused_widget = Focus::Tracklist;
                        // return Some(Message::Playlist(playlist::Message::SelectPlaylist));
                    }
                    Instruction::DeletePlaylist(md) => {
                        md.delete(&self.db_conn);
                        let playlists = PlaylistMd::get_all(&self.db_conn);
                        return Task::new(Message::Playlist(playlist::Message::SetPlaylists(
                            playlists,
                        )));
                    }
                    Instruction::SetMode(mode) => self.mode = mode,
                    Instruction::CreatePlaylist(name) => {
                        match self.mode {
                            Mode::Default => {}
                            Mode::Write => self.mode = Mode::Default,
                        };

                        PlaylistMd::new(name).save(&self.db_conn);
                        let playlists = model::PlaylistMd::get_all(&self.db_conn);
                        return Task::new(Message::Playlist(playlist::Message::SetPlaylists(
                            playlists,
                        )));
                    }
                }
            }
            Instruction::ControlBar(instruction) => {
                use control_bar::Instruction;

                match instruction {
                    Instruction::SetShuffle(shuffle) => {
                        self.shuffle = shuffle.clone();
                        self.tracklist.shuffle = shuffle;
                    }
                    Instruction::SetRepeat(repeat) => {
                        self.repeat = repeat.clone();
                        self.tracklist.repeat = repeat;
                    }
                    Instruction::SetPause(paused) => {
                        self.paused = paused;
                        if self.paused {
                            self.audio_tx.send(Command::Pause).unwrap();
                        } else {
                            self.audio_tx.send(Command::Resume).unwrap();
                        }
                    }
                    Instruction::SeekForward(dur) => self.seek_forward(dur),
                    Instruction::SeekBackward(dur) => self.seek_backward(dur),
                }
            }
        }

        Task::none()
    }

    fn seek_forward(&mut self, duration: Duration) {
        if self.last_seek_timer.elapsed() > Duration::from_millis(50) {
            self.audio_tx
                .send(music::Command::seek_forward(duration))
                .unwrap();
            self.last_seek_timer = Instant::now();
        }
    }

    fn seek_backward(&mut self, duration: Duration) {
        if self.last_seek_timer.elapsed() > Duration::from_millis(50) {
            self.audio_tx
                .send(music::Command::seek_backward(duration))
                .unwrap();
            self.last_seek_timer = Instant::now();
        }
    }
}

impl Widget for &Player {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if self.fallback_render {
            Widget::render(Fallback, area, buf);
        } else {
            self.header.view(buf);

            match self.focused_widget {
                Focus::Tracklist => self.tracklist.view(buf),
                Focus::Playlist => self.playlist.view(buf),
            }

            self.control_bar.view(buf);
        }
    }
}
