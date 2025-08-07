use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{buffer::Buffer, crossterm::event::MouseEvent, layout::Rect};

pub mod control_bar;
pub mod gen_popup;
pub mod header;
pub mod playlist;
pub mod new_playlist_input;
pub mod tracklist;

pub trait Component {
    type Output;
    type Message;
    
    fn area(&self) -> Rect;
    fn resize(&mut self, area: Rect);
    fn view(&self, buffer: &mut Buffer);

    #[allow(unused_variables)]
    fn update(&mut self, message: Self::Message) -> Self::Output;

    #[allow(unused_variables)]
    fn handle_mouse(&self, ev: MouseEvent) -> Option<Self::Message> {
        unimplemented!()
    }

    #[allow(unused_variables)]
    fn handle_input(&self, code: KeyCode, mods: KeyModifiers) -> Option<Self::Message> {
        unimplemented!()
    }
}
