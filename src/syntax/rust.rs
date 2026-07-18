//! Rust 单行高亮：关键字、类型推断、raw 字符串等启发式规则
//!
//! 非 rustc 级语义；属性行整行标 Macro，// 与 /* */ 按 merge 模块切分

use super::merge::{is_screaming_snake, merge_spans, skip_ws};
use super::scan::{match_longest_op, scan_number, scan_string, slice_contains, WordCtx};
use super::{HighlightKind, Span};

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "dyn", "box", "yield", "macro", "become", "async", "try",
    "do", "final", "override", "priv", "typeof", "abstract",
];

const BUILTIN_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "str", "String", "Vec", "Option", "Result", "Box", "Rc", "Arc", "Cell",
    "RefCell", "Cow", "Pin",
];

const TYPE_INTRODUCERS: &[&str] = &["struct", "enum", "trait", "type", "impl", "union"];

const LOGICAL_OPS: &[&str] = &["==", "!=", "<=", ">=", "&&", "||", "=>"];
const ACCESS_OPS: &[&str] = &["->", "::"];
const COMPOUND_OPS: &[&str] = &["+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>="];

#[derive(Debug, Clone, Copy, Default)]
struct RustCtx {
    base: WordCtx,
    after_fn: bool,
    after_binding: bool,
}

pub fn highlight_rust_line(line: &str) -> Vec<Span> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("#![") || trimmed.starts_with("#[") {
        return merge_spans(
            vec![Span {
                start: 0,
                end: line.len(),
                kind: HighlightKind::Macro,
            }],
            line.len(),
        );
    }
    if let Some(idx) = line.find("//") {
        let mut spans = highlight_rust_code(&line[..idx]);
        spans.push(Span {
            start: idx,
            end: line.len(),
            kind: HighlightKind::Comment,
        });
        return merge_spans(spans, line.len());
    }
    if let Some(idx) = line.find("/*") {
        if let Some(rel) = line[idx + 2..].find("*/") {
            let end = idx + 2 + rel + 2;
            let mut spans = highlight_rust_code(&line[..idx]);
            spans.push(Span {
                start: idx,
                end,
                kind: HighlightKind::Comment,
            });
            spans.extend(highlight_rust_code(&line[end..]));
            return merge_spans(spans, line.len());
        }
    }
    merge_spans(highlight_rust_code(line), line.len())
}

fn highlight_rust_code(text: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut i = 0;
    let mut ctx = RustCtx::default();

    while i < text.len() {
        if match_longest_op(text, i, LOGICAL_OPS).is_some_and(|n| {
            spans.push(Span {
                start: i,
                end: i + n,
                kind: HighlightKind::OperatorLogical,
            });
            i += n;
            true
        }) {
            continue;
        }
        if match_longest_op(text, i, ACCESS_OPS).is_some_and(|n| {
            spans.push(Span {
                start: i,
                end: i + n,
                kind: HighlightKind::Operator,
            });
            ctx.base.on_access_op();
            i += n;
            true
        }) {
            continue;
        }
        if match_longest_op(text, i, COMPOUND_OPS).is_some_and(|n| {
            spans.push(Span {
                start: i,
                end: i + n,
                kind: HighlightKind::OperatorLogical,
            });
            i += n;
            true
        }) {
            continue;
        }

        let ch = text[i..].chars().next().unwrap();
        if ch.is_ascii_whitespace() {
            i += ch.len_utf8();
            continue;
        }
        match ch {
            '{' | '}' | ';' | ',' => {
                spans.push(Span {
                    start: i,
                    end: i + ch.len_utf8(),
                    kind: HighlightKind::Punctuation,
                });
                i += ch.len_utf8();
            }
            '(' => {
                spans.push(Span {
                    start: i,
                    end: i + 1,
                    kind: HighlightKind::Punctuation,
                });
                ctx.base.on_open_paren();
                i += 1;
            }
            ')' => {
                spans.push(Span {
                    start: i,
                    end: i + 1,
                    kind: HighlightKind::Punctuation,
                });
                ctx.base.on_close_paren();
                i += 1;
            }
            '[' | ']' => {
                spans.push(Span {
                    start: i,
                    end: i + ch.len_utf8(),
                    kind: HighlightKind::Punctuation,
                });
                i += ch.len_utf8();
            }
            '.' if !text[i..].starts_with("..") => {
                spans.push(Span {
                    start: i,
                    end: i + 1,
                    kind: HighlightKind::Operator,
                });
                ctx.base.on_access_op();
                i += 1;
            }
            'r' if text[i..].starts_with("r#\"") || text[i..].starts_with("r\"") => {
                let (s, ni) = scan_raw_string(text, i);
                spans.extend(s);
                i = ni;
            }
            '"' => {
                let (s, ni) = scan_string(text, i, '"');
                spans.extend(s);
                i = ni;
            }
            '\'' => {
                let (s, ni) = scan_char_or_lifetime(text, i);
                spans.extend(s);
                i = ni;
            }
            '0'..='9' => {
                let (s, ni) = scan_number(text, i);
                spans.push(s);
                i = ni;
            }
            _ if ch.is_ascii_alphabetic() || ch == '_' => {
                let start = i;
                i += ch.len_utf8();
                while i < text.len() {
                    let c = text[i..].chars().next().unwrap();
                    if c.is_ascii_alphanumeric() || c == '_' {
                        i += c.len_utf8();
                    } else {
                        break;
                    }
                }
                let word = &text[start..i];
                let rest = skip_ws(text, i);
                let mut kind = classify_rust_word(word, &ctx, text, rest);
                if text.get(i..).is_some_and(|r| r.starts_with('!')) {
                    kind = HighlightKind::Macro;
                    i += 1;
                }
                if word == "fn" {
                    ctx.after_fn = true;
                } else if word == "let" || word == "const" {
                    ctx.after_binding = true;
                } else if ctx.after_fn && kind != HighlightKind::Keyword {
                    ctx.after_fn = false;
                } else if ctx.after_binding && kind == HighlightKind::Parameter {
                    ctx.after_binding = false;
                }
                if slice_contains(TYPE_INTRODUCERS, word) {
                    ctx.base.on_type_introducer();
                } else if ctx.base.expect_type_name {
                    ctx.base.on_type_name_seen();
                }
                if kind == HighlightKind::Property {
                    ctx.base.after_dot = false;
                }
                spans.push(Span {
                    start,
                    end: i,
                    kind,
                });
            }
            '!' | '=' | '<' | '>' | '&' | '|' | '^' | '~' | '%' | '+' | '-' | '*' | '/' | '@' => {
                let kind = if matches!(ch, '!' | '<' | '>' | '&' | '|') {
                    HighlightKind::OperatorLogical
                } else if ch == '@' {
                    HighlightKind::Macro
                } else {
                    HighlightKind::Punctuation
                };
                spans.push(Span {
                    start: i,
                    end: i + ch.len_utf8(),
                    kind,
                });
                i += ch.len_utf8();
            }
            _ => {
                i += ch.len_utf8();
            }
        }
    }
    spans
}

fn classify_rust_word(word: &str, ctx: &RustCtx, text: &str, rest: usize) -> HighlightKind {
    if matches!(word, "self" | "Self") {
        return HighlightKind::LanguageSpecial;
    }
    if slice_contains(BUILTIN_TYPES, word) {
        return HighlightKind::Type;
    }
    if slice_contains(KEYWORDS, word) {
        return HighlightKind::Keyword;
    }
    if ctx.base.after_dot {
        return HighlightKind::Property;
    }
    if ctx.after_fn {
        return HighlightKind::Function;
    }
    if ctx.base.expect_type_name {
        return HighlightKind::Type;
    }
    if ctx.after_binding {
        return HighlightKind::Parameter;
    }
    if ctx.base.expect_param_name {
        return HighlightKind::Parameter;
    }
    if is_screaming_snake(word) {
        return HighlightKind::Constant;
    }
    let after = text.get(rest..).unwrap_or("");
    if after.starts_with('(') {
        return HighlightKind::Function;
    }
    if after.starts_with('!') {
        return HighlightKind::Macro;
    }
    HighlightKind::Plain
}

fn scan_char_or_lifetime(text: &str, i: usize) -> (Vec<Span>, usize) {
    let start = i;
    let mut j = i + 1;
    while j < text.len() {
        let c = text[j..].chars().next().unwrap();
        j += c.len_utf8();
        if c == '\'' {
            break;
        }
    }
    let inner = &text[start + 1..j.saturating_sub(1)];
    let kind = if inner.starts_with('_')
        || inner
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
    {
        HighlightKind::Type
    } else {
        HighlightKind::String
    };
    (
        vec![Span {
            start,
            end: j,
            kind,
        }],
        j,
    )
}

fn scan_raw_string(text: &str, i: usize) -> (Vec<Span>, usize) {
    let start = i;
    let mut j = i + 2;
    let _ = &mut j;
    if text[i..].starts_with("r#\"") {
        j = i + 3;
        while j + 1 < text.len() {
            if text[j..].starts_with("\"") {
                j += 1;
                break;
            }
            j += 1;
        }
    } else {
        let (s, nj) = scan_string(text, i + 1, '"');
        let mut out = vec![Span {
            start: i,
            end: i + 2,
            kind: HighlightKind::StringDelim,
        }];
        for mut sp in s {
            sp.start += i + 1;
            sp.end += i + 1;
            out.push(sp);
        }
        return (out, nj);
    }
    (
        vec![Span {
            start,
            end: j,
            kind: HighlightKind::String,
        }],
        j,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_keyword_and_comment() {
        let spans = highlight_rust_line("let x = 1; // comment");
        assert!(spans.iter().any(|s| s.kind == HighlightKind::Keyword));
        assert!(spans.iter().any(|s| s.kind == HighlightKind::Parameter));
    }

    #[test]
    fn highlights_fn_name_as_function() {
        let spans = highlight_rust_line("fn fibonacci(n: u32) -> u64 {");
        assert!(spans.iter().any(|s| s.kind == HighlightKind::Function));
    }

    #[test]
    fn highlights_attribute_as_macro() {
        let spans = highlight_rust_line("#[derive(Debug)]");
        assert!(spans.iter().all(|s| s.kind == HighlightKind::Macro));
    }
}
