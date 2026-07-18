//! 查找/替换：跨标签共享的 SearchState，匹配坐标为行内字节列
//!
//! Match.col / len 与 str::find 一致；光标跳转需经 line_byte_col_to_offset
//! replace_all 自后向前替换，避免偏移错位

use regex::RegexBuilder;

use crate::buffer::GapBuffer;

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchOptions {
    pub use_regex: bool,
    pub case_insensitive: bool,
}

/// 单次匹配：row 为逻辑行号，col/len 为该行 UTF-8 字节偏移
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub row: usize,
    pub col: usize,
    pub len: usize,
}

/// 当前标签的搜索会话；matches 在 compile 时按全文重建
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub query: String,
    pub forward: bool,
    pub options: SearchOptions,
    pub matches: Vec<Match>,
    pub current: Option<usize>,
}

impl SearchState {
    pub fn clear(&mut self) {
        self.query.clear();
        self.matches.clear();
        self.current = None;
    }

    pub fn toggle_regex(&mut self) {
        self.options.use_regex = !self.options.use_regex;
    }

    pub fn toggle_case_insensitive(&mut self) {
        self.options.case_insensitive = !self.options.case_insensitive;
    }

    pub fn options_label(&self) -> String {
        let mut parts = Vec::new();
        if self.options.use_regex {
            parts.push("regex");
        }
        if self.options.case_insensitive {
            parts.push("icase");
        }
        if parts.is_empty() {
            "literal".into()
        } else {
            parts.join("+")
        }
    }

    /// 重新扫描 buffer，填充 matches；空 query 时清空
    pub fn compile(&mut self, buffer: &GapBuffer) -> Result<(), String> {
        self.matches.clear();
        self.current = None;
        if self.query.is_empty() {
            return Ok(());
        }
        let text = buffer.as_text();
        if self.options.use_regex {
            self.compile_regex(&text)?;
        } else if self.options.case_insensitive {
            self.compile_literal_icase(&text)?;
        } else {
            self.compile_literal(&text);
        }
        self.set_current_from_direction();
        Ok(())
    }

    fn set_current_from_direction(&mut self) {
        if self.matches.is_empty() {
            self.current = None;
        } else {
            self.current = Some(if self.forward {
                0
            } else {
                self.matches.len() - 1
            });
        }
    }

    fn compile_literal(&mut self, text: &str) {
        for (row, line) in text.lines().enumerate() {
            let mut start = 0usize;
            while let Some(idx) = line[start..].find(&self.query) {
                self.matches.push(Match {
                    row,
                    col: start + idx,
                    len: self.query.len(),
                });
                start += idx + self.query.len().max(1);
            }
        }
    }

    fn compile_literal_icase(&mut self, text: &str) -> Result<(), String> {
        let pattern = regex::escape(&self.query);
        let re = RegexBuilder::new(&pattern)
            .case_insensitive(true)
            .build()
            .map_err(|e| e.to_string())?;
        for (row, line) in text.lines().enumerate() {
            for m in re.find_iter(line) {
                self.matches.push(Match {
                    row,
                    col: m.start(),
                    len: m.end() - m.start(),
                });
            }
        }
        Ok(())
    }

    fn compile_regex(&mut self, text: &str) -> Result<(), String> {
        let re = RegexBuilder::new(&self.query)
            .case_insensitive(self.options.case_insensitive)
            .build()
            .map_err(|e| e.to_string())?;
        for (row, line) in text.lines().enumerate() {
            for m in re.find_iter(line) {
                self.matches.push(Match {
                    row,
                    col: m.start(),
                    len: m.end() - m.start(),
                });
            }
        }
        Ok(())
    }

    pub fn current_match(&self) -> Option<&Match> {
        self.current.and_then(|i| self.matches.get(i))
    }

    /// 按 forward 方向在 matches 中循环
    pub fn next_match(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        let idx = self.current.unwrap_or(0);
        let next = if self.forward {
            (idx + 1) % self.matches.len()
        } else {
            (idx + self.matches.len() - 1) % self.matches.len()
        };
        self.current = Some(next);
    }

    /// 替换当前匹配并重新 compile（偏移可能变化）
    pub fn replace_current(&mut self, buffer: &mut GapBuffer, replacement: &str) -> bool {
        let Some(idx) = self.current else {
            return false;
        };
        let Some(m) = self.matches.get(idx).cloned() else {
            return false;
        };
        self.replace_at(buffer, &m, replacement);
        let _ = self.compile(buffer);
        if self.matches.is_empty() {
            self.current = None;
        } else {
            self.current = Some(idx.min(self.matches.len() - 1));
        }
        true
    }

    /// 全部替换；倒序应用以免 byte offset 漂移
    pub fn replace_all(&mut self, buffer: &mut GapBuffer, replacement: &str) -> usize {
        let _ = self.compile(buffer);
        let mut sorted: Vec<Match> = self.matches.clone();
        // 从文件末尾往前删，保证尚未处理的匹配 offset 仍有效
        sorted.sort_by(|a, b| b.row.cmp(&a.row).then(b.col.cmp(&a.col)));
        let count = sorted.len();
        for m in sorted {
            self.replace_at(buffer, &m, replacement);
        }
        self.current = None;
        let _ = self.compile(buffer);
        count
    }

    fn replace_at(&self, buffer: &mut GapBuffer, m: &Match, replacement: &str) {
        let start = buffer.line_byte_col_to_offset(m.row, m.col);
        let end = start + m.len;
        buffer.delete_byte_range(start, end);
        buffer.insert_str(start, replacement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_matches_literal() {
        let buf = GapBuffer::from_str("fn main\nfn test");
        let mut s = SearchState {
            query: "fn".into(),
            forward: true,
            ..Default::default()
        };
        s.compile(&buf).unwrap();
        assert_eq!(s.matches.len(), 2);
    }

    #[test]
    fn find_matches_regex_icase() {
        let buf = GapBuffer::from_str("Fn main\nfn test");
        let mut s = SearchState {
            query: r"fn".into(),
            options: SearchOptions {
                use_regex: true,
                case_insensitive: true,
            },
            ..Default::default()
        };
        s.compile(&buf).unwrap();
        assert_eq!(s.matches.len(), 2);
    }

    #[test]
    fn find_regex_line_start() {
        let buf = GapBuffer::from_str("#define A 1\nint x;\n#define B 2");
        let mut s = SearchState {
            query: r"^#define".into(),
            options: SearchOptions {
                use_regex: true,
                ..Default::default()
            },
            ..Default::default()
        };
        s.compile(&buf).unwrap();
        assert_eq!(s.matches.len(), 2);
    }

    #[test]
    fn invalid_regex_returns_error() {
        let buf = GapBuffer::from_str("abc");
        let mut s = SearchState {
            query: "(unclosed".into(),
            options: SearchOptions {
                use_regex: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(s.compile(&buf).is_err());
    }

    #[test]
    fn replace_on_crlf_buffer() {
        let mut buf = GapBuffer::from_str("foo\r\nfoo\r\n");
        let mut s = SearchState {
            query: "foo".into(),
            ..Default::default()
        };
        s.compile(&buf).unwrap();
        assert_eq!(s.replace_all(&mut buf, "bar"), 2);
        assert_eq!(buf.as_text(), "bar\r\nbar\r\n");
    }

    #[test]
    fn backward_search_starts_at_last_match() {
        let buf = GapBuffer::from_str("foo bar foo");
        let mut s = SearchState {
            query: "foo".into(),
            forward: false,
            ..Default::default()
        };
        s.compile(&buf).unwrap();
        assert_eq!(s.current, Some(1));
    }

    #[test]
    fn replace_all() {
        let mut buf = GapBuffer::from_str("foo bar foo");
        let mut s = SearchState {
            query: "foo".into(),
            ..Default::default()
        };
        let n = s.replace_all(&mut buf, "baz");
        assert_eq!(n, 2);
        assert_eq!(buf.as_text(), "baz bar baz");
    }

    #[test]
    fn replace_all_utf8_line() {
        let mut buf = GapBuffer::from_str("你好 foo 世界");
        let mut s = SearchState {
            query: "foo".into(),
            ..Default::default()
        };
        let n = s.replace_all(&mut buf, "bar");
        assert_eq!(n, 1);
        assert_eq!(buf.as_text(), "你好 bar 世界");
    }

    #[test]
    fn goto_match_uses_byte_col() {
        let buf = GapBuffer::from_str("你好 fn test");
        let mut s = SearchState {
            query: "fn".into(),
            ..Default::default()
        };
        s.compile(&buf).unwrap();
        let m = s.matches[0].clone();
        let mut cur = crate::cursor::Cursor::new();
        cur.set_from_offset(&buf, buf.line_byte_col_to_offset(m.row, m.col));
        assert_eq!(cur.col, 3);
    }
}
