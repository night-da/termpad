//! 选区 stub：Commit 08 整文件替换为终态 selection.rs

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
        false
    }

    pub fn begin(&mut self, at: Cursor) {
        self.anchor = Some(at);
    }

    pub fn ordered_range(&self, _head: Cursor) -> Option<(Cursor, Cursor)> {
        None
    }

    pub fn cols_on_line(
        &self,
        _row: usize,
        _head: Cursor,
        _line_len: usize,
    ) -> Option<(usize, usize)> {
        None
    }
}
