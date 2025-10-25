use std::cell::Cell;

pub struct VerticalScroll {
    pub y_offset: Cell<usize>,
    pub pos: Cell<usize>,
    pub going_down: Cell<bool>,
}

impl VerticalScroll {
    pub fn new() -> Self {
        VerticalScroll {
            y_offset: Cell::new(0),
            pos: Cell::new(0),
            going_down: Cell::new(true),
        }
    }

    pub fn pos(&self) -> usize {
        self.pos.get()
    }

    pub fn move_up(&self) {
        let pos = self.pos.get();
        self.pos.set(pos.saturating_sub(1));
        self.going_down.set(false);
    }

    pub fn move_down(&self, max_len: usize) {
        let pos = self.pos.get();
        if pos < max_len - 1 {
            self.pos.set(pos + 1);
        }
        self.going_down.set(true);
    }

    pub fn update(&self, visible_height: usize, max_selection: usize) {
        let new_y_offset = self.calc_scroll_offset(
            visible_height,
            self.pos.get(),
            max_selection,
        );

        self.y_offset.set(new_y_offset);

        // if visible_height == 0 {
        //     self.y_offset.set(0);
        // } else {
        //     eprintln!("selection: {selection}, offset: {}",
        // self.y_offset.get());     if new_pos > self.pos.get() {
        //         self.going_down.set(true);
        //     } else if new_y_offset < self.y_offset.get() {
        //         self.going_down.set(false);
        //     }
        //     self.pos.set(new_pos);
        // }
    }

    fn calc_scroll_offset(
        &self,
        visible_height: usize,
        selection: usize,
        max_selection: usize,
    ) -> usize {
        let y_offset = self.y_offset.get();

        if visible_height == 0 {
            return 0;
        }

        // Can be utilized in future if user wants some padding
        let pad = 0;

        if selection >= max_selection {
            return max_selection;
        }

        if self.going_down.get() {
            if selection > visible_height + y_offset - 1 - pad
                && max_selection > y_offset + visible_height
            {
                selection.saturating_sub(visible_height - pad) + 1
            } else {
                y_offset
            }
        } else {
            if selection < y_offset.saturating_add(pad) {
                y_offset.saturating_sub(1)
            } else {
                y_offset
            }
        }
    }
}
