use crate::cursor::Cursor;
use crate::document::Document;
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
    })
}

#[allow(dead_code)]
pub fn debug_log_dimensions(dimensions: &Wh, log_file: &mut File) {
    log_file
        .write(
            format!(
                "Terminal width: {}, Terminal height: {}\n",
                dimensions.width, dimensions.height
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
                "Cursor row: {}, Cursor row relative to document: {}, Cursor column: {}\n",
                cursor.row,
                cursor.row - 2,
                cursor.column
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
