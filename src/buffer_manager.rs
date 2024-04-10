use crate::document::Document;
use crate::editor::Editor;
use std::cell::RefCell;

pub struct BufManager {
    buffers: Vec<RefCell<Document>>,
    active_buffer: usize,
}

impl BufManager {
    pub fn new(file_name: &str, editor_dim: &Editor) -> Self {
        Self {
            buffers: vec![RefCell::new(Document::new(file_name, editor_dim))],
            active_buffer: 0,
        }
    }

    pub fn new_scratch(doc_disp_height: usize) -> Self {
        Self {
            buffers: vec![RefCell::new(Document::new_scratch(doc_disp_height))],
            active_buffer: 0,
        }
    }

    pub fn add_buffer(&mut self, file_name: &str, editor_dim: &Editor) {
        self.buffers
            .push(RefCell::new(Document::new(file_name, editor_dim)));
    }

    pub fn remove_buffer(&mut self) {
        self.buffers.remove(self.active_buffer);
    }
}
