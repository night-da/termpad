//! 单遍扫描辅助（无跨行/文件级启发式）
//!
//! scan_number / scan_string 供 rust 与 cfamily 共用

use super::{HighlightKind, Span};

pub(crate) fn match_longest_op(text: &str, i: usize, ops: &[&str]) -> Option<usize> {
    ops.iter()
        .filter(|op| text[i..].starts_with(**op))
        .map(|op| op.len())
        .max()
}

pub(crate) fn scan_number(text: &str, i: usize) -> (Span, usize) {
    let start = i;
    let mut j = i;
    if text[j..].starts_with("0x") || text[j..].starts_with("0X") {
        j += 2;
        while j < text.len() {
            let c = text[j..].chars().next().unwrap();
            if c.is_ascii_hexdigit() || c == '_' {
                j += c.len_utf8();
            } else {
                break;
            }
        }
    } else {
        j += text[j..].chars().next().unwrap().len_utf8();
        while j < text.len() {
            let c = text[j..].chars().next().unwrap();
            if c.is_ascii_digit() || c == '.' || c == '_' {
                j += c.len_utf8();
            } else if (c == 'e' || c == 'E') && j + 1 < text.len() {
                j += c.len_utf8();
                if text[j..].starts_with('+') || text[j..].starts_with('-') {
                    j += 1;
                }
            } else if c == 'f' || c == 'F' || c == 'L' || c == 'U' || c == 'u' {
                j += c.len_utf8();
                if (c == 'U' || c == 'u')
                    && (text[j..].starts_with('l') || text[j..].starts_with('L'))
                {
                    j += 1;
                }
                break;
            } else {
                break;
            }
        }
    }
    (
        Span {
            start,
            end: j,
            kind: HighlightKind::Number,
        },
        j,
    )
}

pub(crate) fn scan_string(text: &str, i: usize, quote: char) -> (Vec<Span>, usize) {
    let mut spans = Vec::new();
    let q_len = quote.len_utf8();
    spans.push(Span {
        start: i,
        end: i + q_len,
        kind: HighlightKind::StringDelim,
    });
    let mut j = i + q_len;
    while j < text.len() {
        let ch = text[j..].chars().next().unwrap();
        if ch == '\\' {
            let esc = j;
            j += ch.len_utf8();
            if j < text.len() {
                j += text[j..].chars().next().unwrap().len_utf8();
            }
            spans.push(Span {
                start: esc,
                end: j,
                kind: HighlightKind::Escape,
            });
            continue;
        }
        if ch == quote {
            let body_start = i + q_len;
            if j > body_start {
                spans.push(Span {
                    start: body_start,
                    end: j,
                    kind: HighlightKind::String,
                });
            }
            spans.push(Span {
                start: j,
                end: j + q_len,
                kind: HighlightKind::StringDelim,
            });
            return (spans, j + q_len);
        }
        j += ch.len_utf8();
    }
    spans.push(Span {
        start: i + q_len,
        end: j,
        kind: HighlightKind::String,
    });
    (spans, j)
}

/// 单词分类上下文（单遍扫描中更新）
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct WordCtx {
    pub expect_type_name: bool,
    pub expect_param_name: bool,
    pub after_dot: bool,
    pub paren_depth: u16,
}

impl WordCtx {
    pub(crate) fn on_type_introducer(&mut self) {
        self.expect_type_name = true;
    }

    pub(crate) fn on_type_name_seen(&mut self) {
        self.expect_type_name = false;
        if self.paren_depth > 0 {
            self.expect_param_name = true;
        }
    }

    pub(crate) fn on_open_paren(&mut self) {
        self.paren_depth = self.paren_depth.saturating_add(1);
    }

    pub(crate) fn on_close_paren(&mut self) {
        self.paren_depth = self.paren_depth.saturating_sub(1);
        if self.paren_depth == 0 {
            self.expect_param_name = false;
        }
    }

    pub(crate) fn on_access_op(&mut self) {
        self.after_dot = true;
    }
}

pub(crate) fn slice_contains(haystack: &[&str], needle: &str) -> bool {
    haystack.contains(&needle)
}
