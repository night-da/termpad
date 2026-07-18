//! Markdown 单行高亮：标题、引用、粗斜体、链接、代码块围栏等
//!
//! 行内 ** / * / ` / []() 为启发式扫描，非完整 CommonMark 解析

use super::merge::{advance_one_char, merge_spans};
use super::{HighlightKind, Span};

/// ccpp_theme markdown / prose 词法（见 ccpp_theme.json tokenColors）
pub fn highlight_markdown_line(line: &str) -> Vec<Span> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return highlight_heading_line(line);
    }
    if trimmed.starts_with('>') {
        return highlight_blockquote_line(line);
    }
    if is_horizontal_rule(trimmed) {
        return vec![Span {
            start: 0,
            end: line.len(),
            kind: HighlightKind::HorizontalRule,
        }];
    }
    if trimmed.starts_with("```") {
        return vec![Span {
            start: 0,
            end: line.len(),
            kind: HighlightKind::Code,
        }];
    }
    let mut spans = scan_markdown_inline(line);
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        let lead = line.len() - trimmed.len();
        spans.push(Span {
            start: lead,
            end: lead + 2,
            kind: HighlightKind::ListMark,
        });
    }
    merge_spans(spans, line.len())
}

fn highlight_heading_line(line: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let hash_start = i;
    while i < bytes.len() && bytes[i] == b'#' {
        i += 1;
    }
    if i > hash_start {
        spans.push(Span {
            start: hash_start,
            end: i,
            kind: HighlightKind::Heading,
        });
    }
    if i < line.len() {
        let rest = &line[i..];
        let mut inner = scan_markdown_inline(rest);
        for s in &mut inner {
            if s.kind == HighlightKind::Plain {
                s.kind = HighlightKind::Heading;
            }
            s.start += i;
            s.end += i;
        }
        spans.extend(inner);
    }
    merge_spans(spans, line.len())
}

fn highlight_blockquote_line(line: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let trimmed = line.trim_start();
    let offset = line.len() - trimmed.len();
    spans.push(Span {
        start: offset,
        end: offset + 1,
        kind: HighlightKind::Quote,
    });
    if trimmed.len() > 1 {
        let mut inner = scan_markdown_inline(&trimmed[1..]);
        for s in &mut inner {
            if s.kind == HighlightKind::Plain {
                s.kind = HighlightKind::Quote;
            }
            s.start += offset + 1;
            s.end += offset + 1;
        }
        spans.extend(inner);
    }
    merge_spans(spans, line.len())
}

fn is_horizontal_rule(line: &str) -> bool {
    let t = line.trim();
    if t.len() < 3 {
        return false;
    }
    t.chars().all(|c| c == '-' || c == '*' || c == '_')
}

fn scan_markdown_inline(text: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < text.len() {
        debug_assert!(text.is_char_boundary(i));

        if text.as_bytes().get(i) == Some(&b'`') {
            if let Some(rel) = text[i + 1..].find('`') {
                let end = i + 1 + rel + 1;
                spans.push(Span {
                    start: i,
                    end,
                    kind: HighlightKind::Code,
                });
                i = end;
                continue;
            }
        }
        if text[i..].starts_with("**") {
            if let Some(rel) = text[i + 2..].find("**") {
                let end = i + 2 + rel + 2;
                spans.push(Span {
                    start: i,
                    end,
                    kind: HighlightKind::Bold,
                });
                i = end;
                continue;
            }
        }
        if text[i..].starts_with('*') && !text[i..].starts_with("**") {
            if let Some(rel) = text[i + 1..].find('*') {
                let end = i + 1 + rel + 1;
                spans.push(Span {
                    start: i,
                    end,
                    kind: HighlightKind::Italic,
                });
                i = end;
                continue;
            }
        }
        if text.as_bytes().get(i) == Some(&b'[') {
            if let Some(close) = text[i + 1..].find(']') {
                let bracket_end = i + 1 + close + 1;
                if text[bracket_end..].starts_with('(') {
                    if let Some(paren) = text[bracket_end + 1..].find(')') {
                        let end = bracket_end + 1 + paren + 1;
                        spans.push(Span {
                            start: i,
                            end: bracket_end,
                            kind: HighlightKind::Link,
                        });
                        spans.push(Span {
                            start: bracket_end,
                            end,
                            kind: HighlightKind::LinkUrl,
                        });
                        i = end;
                        continue;
                    }
                }
            }
        }
        i = advance_one_char(text, i);
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_heading_and_bold() {
        let h = highlight_markdown_line("## Features");
        assert!(h.iter().any(|s| s.kind == HighlightKind::Heading));
        let b = highlight_markdown_line("- **Editor Core**");
        assert!(b.iter().any(|s| s.kind == HighlightKind::Bold));
        assert!(b.iter().any(|s| s.kind == HighlightKind::ListMark));
    }

    #[test]
    fn chinese_prose_with_link_does_not_panic() {
        let line = "轻量 Rust 终端文本编辑器。参照 [Notepad--](https://github.com/cxasm/notepad--) 常用能力";
        let spans = highlight_markdown_line(line);
        assert!(spans.iter().any(|s| s.kind == HighlightKind::Link));
        assert!(spans.iter().any(|s| s.kind == HighlightKind::LinkUrl));
    }

    #[test]
    fn reference_line_link_colors() {
        let line =
            "- [xenkuo/ccpp_theme](https://github.com/xenkuo/ccpp_theme) — UI/syntax color palette";
        let spans = highlight_markdown_line(line);
        let link = spans
            .iter()
            .find(|s| s.kind == HighlightKind::Link)
            .unwrap();
        assert_eq!(&line[link.start..link.end], "[xenkuo/ccpp_theme]");
        let url = spans
            .iter()
            .find(|s| s.kind == HighlightKind::LinkUrl)
            .unwrap();
        assert!(line[url.start..url.end].starts_with("(https://"));
    }

    #[test]
    fn highlight_entire_readme() {
        let text = include_str!("../../README.md");
        for line in text.lines() {
            let _ = highlight_markdown_line(line);
        }
    }
}
