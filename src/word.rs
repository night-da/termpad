//! Normal 模式下光标处单词高亮
//!
//! 取词按字符列扫描；全文查找用字节 find，结果写入 crate::search::Match

use crate::buffer::GapBuffer;
use crate::cursor::Cursor;
use crate::search::Match;

/// 取光标处完整单词（ASCII 字母数字与 _）；非词上返回 None
pub fn word_at_cursor(buffer: &GapBuffer, cursor: &Cursor) -> Option<String> {
    let line = buffer.line(cursor.row)?;
    let chars: Vec<char> = line.chars().collect();
    if cursor.col >= chars.len() || !is_word_char(chars[cursor.col]) {
        return None;
    }
    let mut start = cursor.col;
    let mut end = cursor.col + 1;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }
    Some(chars[start..end].iter().collect())
}

/// 全文查找 word 出现位置；col/len 为字节，与 SearchState 一致
pub fn find_word_occurrences(buffer: &GapBuffer, word: &str) -> Vec<Match> {
    if word.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let text = buffer.as_text();
    for (row, line) in text.lines().enumerate() {
        let mut start = 0usize;
        while let Some(idx) = line[start..].find(word) {
            let col = start + idx;
            let boundary_ok = is_boundary(line, col, word.len());
            if boundary_ok {
                out.push(Match {
                    row,
                    col,
                    len: word.len(),
                });
            }
            start = col + word.len().max(1);
        }
    }
    out
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_boundary(line: &str, col: usize, len: usize) -> bool {
    let before = col == 0 || !line[..col].chars().last().is_some_and(is_word_char);
    let after_idx = col + len;
    let after =
        after_idx >= line.len() || !line[after_idx..].chars().next().is_some_and(is_word_char);
    before && after
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_word_matches() {
        let buf = GapBuffer::from_str("fn main\nfn test");
        let hits = find_word_occurrences(&buf, "fn");
        assert_eq!(hits.len(), 2);
    }
}
