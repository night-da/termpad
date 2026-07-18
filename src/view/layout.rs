//! 视口映射（Commit 08 增加 cursor_for_drag 与 drag 测试）

use crate::buffer::GapBuffer;
use crate::cursor::Cursor;
use crate::document::Document;
use crate::fold::FoldState;

pub fn visible_line_indices(buffer: &GapBuffer, folds: &FoldState) -> Vec<usize> {
    (0..buffer.line_count())
        .filter(|&r| !folds.is_hidden(r))
        .collect()
}

pub fn screen_to_cursor(doc: &Document, x: u16, y: u16) -> Option<Cursor> {
    let area = doc.view.layout.text;
    if area.width == 0 || area.height == 0 {
        return None;
    }
    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }
    let display_row = (y - area.y) as usize;
    let visible_map = visible_line_indices(&doc.buffer, &doc.folds);
    let logical_row = *visible_map.get(doc.view.scroll_row + display_row)?;
    let rel_x = (x - area.x) as usize;
    let line = doc.buffer.line(logical_row)?;
    let col = rel_x.min(line.chars().count());
    Some(Cursor {
        row: logical_row,
        col,
    })
}
