//! 光标 (行, 列)，列单位为字符，与搜索匹配的字节偏移不同

use crate::buffer::GapBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { row: 0, col: 0 }
    }

    /// 限制 row/col 在 buffer 合法范围内（列按字符数）
    pub fn clamp(&mut self, buffer: &GapBuffer) {
        let max_row = buffer.line_count().saturating_sub(1);
        self.row = self.row.min(max_row);
        let max_col = buffer.line_len(self.row);
        self.col = self.col.min(max_col);
    }

    /// 当前光标在全文中的 UTF-8 字节偏移
    pub fn offset(&self, buffer: &GapBuffer) -> usize {
        buffer.position_to_offset(self.row, self.col)
    }

    /// 从字节偏移还原 row/col 并 clamp
    pub fn set_from_offset(&mut self, buffer: &GapBuffer, offset: usize) {
        let (row, col) = buffer.offset_to_position(offset);
        self.row = row;
        self.col = col;
        self.clamp(buffer);
    }

    /// 列内左移；行首时跳到上一行末尾
    pub fn move_left(&mut self, buffer: &GapBuffer) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.col = buffer.line_len(self.row);
        }
    }
    /// 列内右移；行尾时跳到下一行行首
    pub fn move_right(&mut self, buffer: &GapBuffer) {
        let line_len = buffer.line_len(self.row);
        if self.col < line_len {
            self.col += 1;
        } else if self.row + 1 < buffer.line_count() {
            self.row += 1;
            self.col = 0;
        }
    }
    /// 上一行；列超出新行长度时 clamp 到行尾
    pub fn move_up(&mut self, buffer: &GapBuffer) {
        if self.row > 0 {
            self.row -= 1;
            self.clamp(buffer);
        }
    }
    /// 下一行；列超出新行长度时 clamp 到行尾
    pub fn move_down(&mut self, buffer: &GapBuffer) {
        if self.row + 1 < buffer.line_count() {
            self.row += 1;
            self.clamp(buffer);
        }
    }
    pub fn move_home(&mut self) {
        self.col = 0;
    }

    pub fn move_end(&mut self, buffer: &GapBuffer) {
        self.col = buffer.line_len(self.row);
    }

    /// 翻页只改行号；调用方需 clamp（与 move_up 不同）
    pub fn page_up(&mut self, delta: usize) {
        self.row = self.row.saturating_sub(delta);
    }

    pub fn page_down(&mut self, buffer: &GapBuffer, delta: usize) {
        let max_row = buffer.line_count().saturating_sub(1);
        self.row = (self.row + delta).min(max_row);
    }
}
impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_to_line_end() {
        let buf = GapBuffer::from_str("abc\ndef");
        let mut cur = Cursor { row: 0, col: 99 };
        cur.clamp(&buf);
        assert_eq!(cur.col, 3);
    }
}
