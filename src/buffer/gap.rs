//! Gap Buffer：连续字节 + 可滑动 gap 区间，单字符插入/删除均摊 O(1)
//!
//! 读路径：as_text / line 等会 O(n) 重建全文（见架构文档「已知局限」）
//! 坐标：position_to_offset 用字符列；搜索相关接口用字节列

const INITIAL_CAPACITY: usize = 64;
const GROW_FACTOR: usize = 2;

// 内部以 [gap_start, gap_end) 标记未使用的 gap 槽位

#[derive(Debug, Clone)]
pub struct GapBuffer {
    storage: Vec<u8>,
    gap_start: usize,
    gap_end: usize,
}

impl Default for GapBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl GapBuffer {
    pub fn new() -> Self {
        Self {
            storage: vec![0; INITIAL_CAPACITY],
            gap_start: 0,
            gap_end: INITIAL_CAPACITY,
        }
    }

    pub fn from_str(text: &str) -> Self {
        let mut buf = Self::new();
        buf.insert_str(0, text);
        buf
    }

    pub fn len(&self) -> usize {
        self.storage.len() - self.gap_len()
    }

    /// 全文 UTF-8 字符串；每次调用 O(n) 拷贝 gap 两侧
    pub fn as_text(&self) -> String {
        let mut out = Vec::with_capacity(self.len());
        out.extend_from_slice(&self.storage[..self.gap_start]);
        out.extend_from_slice(&self.storage[self.gap_end..]);
        String::from_utf8(out).unwrap_or_default()
    }

    pub fn line_count(&self) -> usize {
        self.as_text().lines().count().max(1)
    }

    pub fn line(&self, row: usize) -> Option<String> {
        self.as_text().lines().nth(row).map(str::to_owned)
    }

    pub fn line_len(&self, row: usize) -> usize {
        self.line(row).map(|l| l.chars().count()).unwrap_or(0)
    }

    /// (row, col) → 字节偏移；col 为字符列
    pub fn position_to_offset(&self, row: usize, col: usize) -> usize {
        let text = self.as_text();
        let mut offset = 0usize;
        for (i, line) in text.lines().enumerate() {
            if i == row {
                let byte_col = line.chars().take(col).map(|c| c.len_utf8()).sum::<usize>();
                return offset + byte_col;
            }
            offset += line.len() + 1;
            // 逻辑行以 \n 分隔
        }
        offset
    }

    /// 字节偏移 → (row, 字符列)
    pub fn offset_to_position(&self, offset: usize) -> (usize, usize) {
        let text = self.as_text();
        let clamped = offset.min(text.len());
        let before = &text[..clamped];
        let row = before.matches('\n').count();
        let col = before.rsplit('\n').next().unwrap_or("").chars().count();
        (row, col)
    }

    /// 搜索 Match 专用：col 为行内字节列，非字符列
    pub fn line_byte_col_to_offset(&self, row: usize, byte_col: usize) -> usize {
        let text = self.as_text();
        let mut offset = 0usize;
        for (i, line) in text.lines().enumerate() {
            if i == row {
                return offset + byte_col.min(line.len());
            }
            offset += line.len() + 1;
        }
        offset
    }

    pub fn insert_str(&mut self, mut pos: usize, s: &str) {
        for ch in s.chars() {
            self.insert_char(pos, ch);
            pos += ch.len_utf8();
        }
    }

    pub fn insert_char(&mut self, pos: usize, ch: char) {
        let mut encoded = [0u8; 4];
        let bytes = ch.encode_utf8(&mut encoded).as_bytes();
        self.move_gap(pos);
        self.ensure_gap(bytes.len());
        for &b in bytes {
            self.storage[self.gap_start] = b;
            self.gap_start += 1;
        }
    }

    pub fn delete_char(&mut self, pos: usize) {
        if pos >= self.len() {
            return;
        }
        self.move_gap(pos);
        let byte = self.storage[self.gap_end];
        let char_len = utf8_char_len(byte);
        self.gap_end += char_len;
    }

    pub fn delete_char_before(&mut self, pos: usize) {
        if pos == 0 {
            return;
        }
        self.delete_char(pos - utf8_char_len_at(self, pos - 1));
    }

    // 按字节区间删除；重建 buffer 而非 gap 内删，以保证 UTF-8 边界且实现简单
    pub fn delete_byte_range(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }
        let text = self.as_text();
        let end = end.min(text.len());
        let start = start.min(end);
        if start >= end {
            return;
        }
        if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
            return;
        }
        let mut new_text = String::with_capacity(text.len() - (end - start));
        new_text.push_str(&text[..start]);
        new_text.push_str(&text[end..]);
        *self = GapBuffer::from_str(&new_text);
    }

    pub fn insert_newline(&mut self, pos: usize) {
        self.insert_char(pos, '\n');
    }

    fn gap_len(&self) -> usize {
        self.gap_end - self.gap_start
    }

    fn ensure_gap(&mut self, needed: usize) {
        while self.gap_len() < needed {
            self.grow();
        }
    }

    fn grow(&mut self) {
        let new_cap = (self.storage.len() * GROW_FACTOR).max(8);
        let mut new_storage = vec![0; new_cap];
        let before = self.gap_start;
        let after_len = self.storage.len() - self.gap_end;
        new_storage[..before].copy_from_slice(&self.storage[..before]);
        let new_gap_end = new_cap - after_len;
        new_storage[new_gap_end..].copy_from_slice(&self.storage[self.gap_end..]);
        self.storage = new_storage;
        self.gap_end = new_gap_end;
    }

    // 将 gap 移到逻辑位置 pos，使后续插入/删除发生在 gap 左缘
    fn move_gap(&mut self, pos: usize) {
        let pos = pos.min(self.len());
        if pos < self.gap_start {
            let shift = self.gap_start - pos;
            for i in (0..shift).rev() {
                self.storage[self.gap_end - 1 - i] = self.storage[self.gap_start - 1 - i];
            }
            self.gap_start -= shift;
            self.gap_end -= shift;
        } else {
            let target = pos;
            let shift = target - self.gap_start;
            for i in 0..shift {
                self.storage[self.gap_start + i] = self.storage[self.gap_end + i];
            }
            self.gap_start += shift;
            self.gap_end += shift;
        }
    }
}

fn utf8_char_len(first: u8) -> usize {
    if first & 0b1000_0000 == 0 {
        1
    } else if first & 0b1110_0000 == 0b1100_0000 {
        2
    } else if first & 0b1111_0000 == 0b1110_0000 {
        3
    } else {
        4
    }
}

fn utf8_char_len_at(buf: &GapBuffer, pos: usize) -> usize {
    let text = buf.as_text();
    let b = text.as_bytes().get(pos).copied().unwrap_or(0);
    utf8_char_len(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_read_line() {
        let mut buf = GapBuffer::new();
        buf.insert_str(0, "fn main() {\n}\n");
        assert_eq!(buf.line(0).as_deref(), Some("fn main() {"));
        assert_eq!(buf.line(1).as_deref(), Some("}"));
    }

    #[test]
    fn delete_char() {
        let mut buf = GapBuffer::from_str("abc");
        buf.delete_char(1);
        assert_eq!(buf.as_text(), "ac");
    }

    #[test]
    fn position_roundtrip() {
        let buf = GapBuffer::from_str("hello\nworld");
        let off = buf.position_to_offset(1, 2);
        assert_eq!(buf.offset_to_position(off), (1, 2));
    }

    #[test]
    fn insert_in_middle() {
        let mut buf = GapBuffer::from_str("ace");
        buf.insert_char(1, 'b');
        assert_eq!(buf.as_text(), "abce");
    }

    #[test]
    fn delete_byte_range_utf8() {
        let mut buf = GapBuffer::from_str("你好 world");
        buf.delete_byte_range(0, 6);
        assert_eq!(buf.as_text(), " world");
    }
}
