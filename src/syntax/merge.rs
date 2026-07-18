//! span 合并与 UTF-8 安全辅助
//!
//! merge_spans 排序后补 Plain 空隙；char_at / find_line_comment 避免在字符串内误切分

use super::{HighlightKind, Span};

/// 合并重叠 span 并在空隙处补 Plain；保证 [0, line_len) 全覆盖
pub(crate) fn merge_spans(mut spans: Vec<Span>, line_len: usize) -> Vec<Span> {
    if spans.is_empty() {
        return vec![Span {
            start: 0,
            end: line_len,
            kind: HighlightKind::Plain,
        }];
    }
    spans.sort_by_key(|s| s.start);
    let mut merged = Vec::new();
    let mut cursor = 0;
    for span in spans {
        if span.start > cursor {
            merged.push(Span {
                start: cursor,
                end: span.start,
                kind: HighlightKind::Plain,
            });
        }
        merged.push(span.clone());
        cursor = cursor.max(span.end);
    }
    if cursor < line_len {
        merged.push(Span {
            start: cursor,
            end: line_len,
            kind: HighlightKind::Plain,
        });
    }
    merged
}

#[inline]
pub(crate) fn char_at(text: &str, i: usize) -> Option<(char, usize)> {
    text.get(i..)?.chars().next().map(|c| (c, c.len_utf8()))
}

pub(crate) fn advance_one_char(text: &str, i: usize) -> usize {
    if i >= text.len() {
        return text.len();
    }
    let step = text[i..].chars().next().map_or(1, |c| c.len_utf8());
    (i + step).min(text.len())
}

pub(crate) fn skip_ws(text: &str, i: usize) -> usize {
    let mut i = i;
    while let Some((ch, len)) = char_at(text, i) {
        if ch.is_ascii_whitespace() {
            i += len;
        } else {
            break;
        }
    }
    i
}

pub(crate) fn peek_word(text: &str, i: usize) -> Option<(usize, usize, &str)> {
    let start = skip_ws(text, i);
    if start >= text.len() {
        return None;
    }
    let ch = text[start..].chars().next()?;
    if !ch.is_ascii_alphabetic() && ch != '_' {
        return None;
    }
    let mut end = start + ch.len_utf8();
    while end < text.len() {
        let Some((c, len)) = char_at(text, end) else {
            break;
        };
        if c.is_ascii_alphanumeric() || c == '_' {
            end += len;
        } else {
            break;
        }
    }
    Some((start, end, &text[start..end]))
}

pub(crate) fn is_screaming_snake(word: &str) -> bool {
    !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && word.chars().any(|c| c.is_ascii_uppercase())
}

/// 查找 // 行注释起点（跳过双引号字符串内的 //）
pub(crate) fn find_line_comment(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut in_string = false;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'"' {
            in_string = !in_string;
        } else if !in_string && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            return Some(i);
        }
        i += 1;
    }
    None
}

pub(crate) fn offset_spans(mut spans: Vec<Span>, delta: usize) -> Vec<Span> {
    for s in &mut spans {
        s.start += delta;
        s.end += delta;
    }
    spans
}
