use crate::cursor::Cursor;
use crate::document::Document;
use crate::editor::Editor;
use crate::gapbuf::GapBuf;
use crate::term::Wh;
use std::fs::File;
use std::io::Write;

#[allow(dead_code)]
pub fn debug_log_message(message: String, log_file: &mut File) {
    log_file.write(message.as_bytes()).unwrap();
}

#[allow(dead_code)]
pub fn debug_log_document(document: &Document, log_file: &mut File) {
    document.lines.iter().for_each(|l| {
        log_file
            .write(format!("Line indices: {:?}, String content: {}\n", l.0, l.1).as_bytes())
            .unwrap();
    });

    log_file
        .write(format!("Visible lines: {:?}\n", document.visible_rows).as_bytes())
        .unwrap();
}

#[allow(dead_code)]
pub fn debug_log_dimensions(dimensions: &Wh, editor_dim: &Editor, log_file: &mut File) {
    log_file
        .write(
            format!(
                "Terminal width: {}, Terminal height: {}\nEditor bottom: {}, Editor width: {}, Editor height: {}, Mode row: {}, Command row: {}\n",
                dimensions.width, dimensions.height, editor_dim.editor_bottom, editor_dim.editor_width, editor_dim.editor_height, editor_dim.mode_row, editor_dim.command_row
            )
            .as_bytes(),
        )
        .unwrap();
}

#[allow(dead_code)]
pub fn debug_log_cursor(cursor: &Cursor, log_file: &mut File) {
    log_file
        .write(
            format!(
                "Cursor row: {}, Cursor column: {}\nCursor row in doc: {}, Cursor column in doc: {}\n",
                cursor.row,
                cursor.column,
                cursor.doc_row,
                cursor.doc_column,
            )
            .as_bytes(),
        )
        .unwrap();
}

#[allow(dead_code)]
pub fn debug_log_gapbuffer(gap_buf: &GapBuf, log_file: &mut File) {
    log_file
        .write(format!("Lhs: {:?}, Rhs: {:?}\n", gap_buf.lhs, gap_buf.rhs).as_bytes())
        .unwrap();
}
