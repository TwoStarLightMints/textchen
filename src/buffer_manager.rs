use crate::document::Document;
use crate::editor::Editor;
use std::cell::RefCell;

pub struct BufManager {
    buffers: Vec<RefCell<Document>>,
    active_buffer: usize,
}

impl BufManager {
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
            active_buffer: 0,
        }
    }
    pub fn add_buffer(&mut self, file_name: &str, editor_dim: &Editor) {
        if self.buffers.len() == 0 {
            self.buffers
                .push(RefCell::new(Document::new(file_name, editor_dim)));
        } else {
            self.buffers
                .push(RefCell::new(Document::new(file_name, editor_dim)));

            self.active_buffer = self.buffers.len() - 1;
        }
    }

    pub fn remove_buffer(&mut self) {
        self.buffers.remove(self.active_buffer);

        if self.active_buffer != 0 {
            self.active_buffer -= 1;
        }
    }
}
