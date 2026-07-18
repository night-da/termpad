//! 文本选区：锚点 + 活动端（head = 当前光标），坐标均为字符列

use crate::cursor::Cursor;

#[derive(Debug, Clone, Copy, Default)]
pub struct Selection {
    pub anchor: Option<Cursor>,
}

impl Selection {
    pub fn clear(&mut self) {
        self.anchor = None;
    }

    pub fn is_active(&self) -> bool {
        self.anchor.is_some()
    }

    pub fn begin(&mut self, at: Cursor) {
        self.anchor = Some(at);
    }

    /// 按文档顺序返回 anchor 与 head 之间的 (start, end) 光标对
    pub fn ordered_range(&self, head: Cursor) -> Option<(Cursor, Cursor)> {
        let anchor = self.anchor?;
        Some(order_cursors(anchor, head))
    }

    /// 该行与选区相交时返回字符列 [start, end)；中间整行则为 [0, line_len)
    pub fn cols_on_line(
        &self,
        row: usize,
        head: Cursor,
        line_len: usize,
    ) -> Option<(usize, usize)> {
        let (start, end) = self.ordered_range(head)?;
        if row < start.row || row > end.row {
            return None;
        }
        let col_start = if row == start.row { start.col } else { 0 };
        let col_end = if row == end.row { end.col } else { line_len };
        // 反向拖选且仅一行时 col_start > col_end，中间行不会出现此情况
        if col_start >= col_end && row != start.row {
            return None;
        }
        Some((
            col_start.min(line_len),
            col_end.min(line_len).max(col_start),
        ))
    }
}

fn order_cursors(a: Cursor, b: Cursor) -> (Cursor, Cursor) {
    if a.row < b.row || (a.row == b.row && a.col <= b.col) {
        (a, b)
    } else {
        (b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cols_on_middle_line_selects_whole_line() {
        let mut sel = Selection::default();
        sel.begin(Cursor { row: 0, col: 2 });
        let head = Cursor { row: 2, col: 5 };
        assert_eq!(sel.cols_on_line(1, head, 20), Some((0, 20)));
    }

    #[test]
    fn cols_on_partial_first_line() {
        let mut sel = Selection::default();
        sel.begin(Cursor { row: 0, col: 3 });
        let head = Cursor { row: 0, col: 7 };
        assert_eq!(sel.cols_on_line(0, head, 10), Some((3, 7)));
    }
}

#[cfg(test)]
mod document_selection_tests {
    use super::super::buffer::GapBuffer;
    use super::super::cursor::Cursor;
    use super::super::document::Document;

    #[test]
    fn delete_selection_removes_text() {
        let mut doc = Document::new_empty(None);
        doc.buffer = GapBuffer::from_str("hello world");
        doc.selection.begin(Cursor { row: 0, col: 0 });
        doc.cursor = Cursor { row: 0, col: 5 };
        assert!(doc.delete_selection());
        assert_eq!(doc.buffer.as_text(), " world");
        assert_eq!(doc.cursor, Cursor { row: 0, col: 0 });
    }
}
