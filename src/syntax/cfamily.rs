//! C/C++ 单行高亮，含预处理、标签、块注释跨行状态
//!
//! CfamilyHighlightState 由 view/text 在渲染前从文件头预热到视口首行

use super::merge::{
    advance_one_char, find_line_comment, is_screaming_snake, merge_spans, offset_spans, peek_word,
    skip_ws,
};
use super::{HighlightKind, Span};

const C_KEYWORDS: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else",
    "enum", "extern", "float", "for", "goto", "if", "inline", "int", "long", "register", "return",
    "short", "signed", "sizeof", "static", "struct", "switch", "typedef", "union", "unsigned",
    "void", "volatile", "while",
];

const CPP_KEYWORDS: &[&str] = &[
    "alignas",
    "alignof",
    "asm",
    "bool",
    "catch",
    "class",
    "const_cast",
    "constexpr",
    "consteval",
    "constinit",
    "decltype",
    "delete",
    "dynamic_cast",
    "explicit",
    "export",
    "false",
    "friend",
    "mutable",
    "namespace",
    "new",
    "noexcept",
    "nullptr",
    "operator",
    "private",
    "protected",
    "public",
    "reinterpret_cast",
    "static_assert",
    "static_cast",
    "template",
    "this",
    "thread_local",
    "true",
    "try",
    "typeid",
    "typename",
    "using",
    "virtual",
    "wchar_t",
];

const BUILTIN_TYPES: &[&str] = &[
    "void", "char", "short", "int", "long", "float", "double", "signed", "unsigned", "bool",
    "size_t", "wchar_t", "int8_t", "int16_t", "int32_t", "int64_t", "uint8_t", "uint16_t",
    "uint32_t", "uint64_t",
];

const TYPE_INTRODUCERS: &[&str] = &["struct", "class", "enum", "union", "typedef"];

const CALL_KEYWORD_BLOCKLIST: &[&str] = &[
    "if", "for", "while", "switch", "catch", "return", "sizeof", "new", "delete",
];

const LOGICAL_OPS: &[&str] = &["==", "!=", "<=", ">=", "&&", "||", "<<", ">>"];
const ACCESS_OPS: &[&str] = &["->", "::", ".*", "->*"];

const COMPOUND_OPS: &[&str] = &["+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>="];

/// 跨行传递块注释状态（/* ... */）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CfamilyHighlightState {
    pub in_block_comment: bool,
}

/// 按单行内容推进块注释状态（视口预热用）
pub fn advance_block_comment_state(line: &str, state: &mut CfamilyHighlightState) {
    let mut i = 0;
    while i < line.len() {
        if state.in_block_comment {
            if let Some(rel) = line[i..].find("*/") {
                i += rel + 2;
                state.in_block_comment = false;
            } else {
                return;
            }
        } else {
            let tail = &line[i..];
            if find_line_comment(tail).is_some() {
                return;
            }
            if let Some(rel) = tail.find("/*") {
                i += rel + 2;
                if tail[rel + 2..].find("*/").is_none() {
                    state.in_block_comment = true;
                    return;
                }
            } else {
                break;
            }
        }
    }
}

pub fn highlight_c_line(line: &str) -> Vec<Span> {
    let mut state = CfamilyHighlightState::default();
    highlight_c_line_with_state(line, &mut state)
}

pub fn highlight_c_line_with_state(line: &str, state: &mut CfamilyHighlightState) -> Vec<Span> {
    highlight_cfamily_line(line, false, state)
}

pub fn highlight_cpp_line(line: &str) -> Vec<Span> {
    let mut state = CfamilyHighlightState::default();
    highlight_cpp_line_with_state(line, &mut state)
}

pub fn highlight_cpp_line_with_state(line: &str, state: &mut CfamilyHighlightState) -> Vec<Span> {
    highlight_cfamily_line(line, true, state)
}

fn highlight_cfamily_line(line: &str, cpp: bool, state: &mut CfamilyHighlightState) -> Vec<Span> {
    let mut at = 0usize;
    let mut spans = Vec::new();

    if state.in_block_comment {
        if let Some(rel) = line.find("*/") {
            let end = rel + 2;
            spans.push(Span {
                start: 0,
                end,
                kind: HighlightKind::Comment,
            });
            state.in_block_comment = false;
            at = end;
        } else {
            return merge_spans(
                vec![Span {
                    start: 0,
                    end: line.len(),
                    kind: HighlightKind::Comment,
                }],
                line.len(),
            );
        }
    }

    if at >= line.len() {
        return merge_spans(spans, line.len());
    }

    let tail = &line[at..];
    if is_label_line(tail) {
        let mut label = highlight_label_line(tail);
        for s in &mut label {
            s.start += at;
            s.end += at;
        }
        spans.extend(label);
        return merge_spans(spans, line.len());
    }

    let trimmed = tail.trim_start();
    let trim_lead = tail.len() - trimmed.len();
    if trimmed.starts_with('#') {
        let mut pp = highlight_preprocessor_line(tail);
        for s in &mut pp {
            s.start += at + trim_lead;
            s.end += at + trim_lead;
        }
        spans.extend(pp);
        return merge_spans(spans, line.len());
    }

    if let Some(rel) = find_line_comment(tail) {
        let abs = at + rel;
        spans.extend(offset_spans(
            highlight_cfamily_code_with_blocks(&line[at..abs], cpp, state),
            at,
        ));
        spans.push(Span {
            start: abs,
            end: line.len(),
            kind: HighlightKind::Comment,
        });
        return merge_spans(spans, line.len());
    }

    spans.extend(offset_spans(
        highlight_cfamily_code_with_blocks(&line[at..], cpp, state),
        at,
    ));
    merge_spans(spans, line.len())
}

fn highlight_cfamily_code_with_blocks(
    text: &str,
    cpp: bool,
    state: &mut CfamilyHighlightState,
) -> Vec<Span> {
    if let Some(idx) = text.find("/*") {
        let mut spans = highlight_cfamily_code_for_mode(&text[..idx], cpp);
        let after = idx + 2;
        if let Some(rel) = text[after..].find("*/") {
            let end = after + rel + 2;
            spans.push(Span {
                start: idx,
                end,
                kind: HighlightKind::Comment,
            });
            spans.extend(highlight_cfamily_code_with_blocks(&text[end..], cpp, state));
            return spans;
        }
        spans.push(Span {
            start: idx,
            end: text.len(),
            kind: HighlightKind::Comment,
        });
        state.in_block_comment = true;
        return spans;
    }
    highlight_cfamily_code_for_mode(text, cpp)
}

fn is_label_line(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() || t.starts_with('#') || t.starts_with("//") {
        return false;
    }
    if let Some(colon) = t.find(':') {
        let head = t[..colon].trim();
        !head.is_empty()
            && head.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && !head.contains(' ')
    } else {
        false
    }
}

fn highlight_label_line(line: &str) -> Vec<Span> {
    let trimmed = line.trim_start();
    let offset = line.len() - trimmed.len();
    let colon = trimmed.find(':').unwrap_or(trimmed.len());
    let mut spans = vec![Span {
        start: offset,
        end: offset + colon,
        kind: HighlightKind::Label,
    }];
    if colon + 1 < trimmed.len() {
        spans.extend(highlight_cfamily_code(&trimmed[colon + 1..], C_KEYWORDS));
        for s in &mut spans {
            if s.start > offset + colon {
                s.start += offset + colon + 1;
                s.end += offset + colon + 1;
            }
        }
    }
    merge_spans(spans, line.len())
}

fn highlight_preprocessor_line(line: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut i = 0;
    let mut saw_define = false;
    while i < line.len() {
        let b = line.as_bytes().get(i).copied();
        if b.is_some_and(|b| b.is_ascii_whitespace()) {
            i += 1;
            continue;
        }
        if b == Some(b'#') {
            let start = i;
            i += 1;
            while i < line.len() && line.as_bytes()[i].is_ascii_alphabetic() {
                i += 1;
            }
            let directive = &line[start..i];
            saw_define = directive == "#define";
            spans.push(Span {
                start,
                end: i,
                kind: HighlightKind::Macro,
            });
            continue;
        }
        if saw_define {
            if let Some((ws, we, _word)) = peek_word(line, i) {
                spans.push(Span {
                    start: ws,
                    end: we,
                    kind: HighlightKind::Macro,
                });
                saw_define = false;
                i = we;
                continue;
            }
        }
        if b == Some(b'"') {
            let (s, ni) = scan_string_spans(line, i, '"');
            spans.extend(s);
            i = ni;
            continue;
        }
        if b == Some(b'<') {
            let start = i;
            i += 1;
            while i < line.len() && line.as_bytes()[i] != b'>' {
                i += 1;
            }
            if i < line.len() {
                i += 1;
            }
            spans.push(Span {
                start,
                end: i,
                kind: HighlightKind::String,
            });
            continue;
        }
        i = advance_one_char(line, i);
    }
    merge_spans(spans, line.len())
}

fn highlight_cfamily_code(text: &str, keywords: &[&str]) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut i = 0;
    let mut expect_type_name = false;
    let line_has_static = text.contains("static");
    let line_has_const_global = !text.contains('{') && text.trim_end().ends_with(';');

    while i < text.len() {
        if try_match_ops(text, i, LOGICAL_OPS, HighlightKind::OperatorLogical).is_some_and(|n| {
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
        if try_match_ops(text, i, ACCESS_OPS, HighlightKind::Operator).is_some_and(|n| {
            spans.push(Span {
                start: i,
                end: i + n,
                kind: HighlightKind::Operator,
            });
            i += n;
            true
        }) {
            continue;
        }
        if try_match_ops(text, i, COMPOUND_OPS, HighlightKind::OperatorLogical).is_some_and(|n| {
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
        if matches!(ch, '{' | '}' | '(' | ')' | '[' | ']' | ';' | ',') {
            spans.push(Span {
                start: i,
                end: i + ch.len_utf8(),
                kind: HighlightKind::Punctuation,
            });
            i += ch.len_utf8();
            continue;
        }
        if ch == '.' && !text[i..].starts_with("..") {
            spans.push(Span {
                start: i,
                end: i + 1,
                kind: HighlightKind::Operator,
            });
            i += 1;
            continue;
        }
        if ch == '"' || ch == '\'' {
            let (s, ni) = scan_string_spans(text, i, ch);
            spans.extend(s);
            i = ni;
            continue;
        }
        if ch.is_ascii_digit() {
            let start = i;
            i += ch.len_utf8();
            while i < text.len() {
                let c = text[i..].chars().next().unwrap();
                if c.is_ascii_digit() || c == '.' || matches!(c, 'x' | 'X' | 'a' | 'b' | 'A' | 'B')
                {
                    i += c.len_utf8();
                } else {
                    break;
                }
            }
            spans.push(Span {
                start,
                end: i,
                kind: HighlightKind::Number,
            });
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
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
            let kind = classify_word(
                word,
                keywords,
                expect_type_name,
                line_has_static,
                line_has_const_global,
                text,
                rest,
            );
            expect_type_name = TYPE_INTRODUCERS.contains(&word);
            spans.push(Span {
                start,
                end: i,
                kind,
            });
            continue;
        }
        i += ch.len_utf8();
    }
    spans
}

fn classify_word(
    word: &str,
    keywords: &[&str],
    expect_type_name: bool,
    line_has_static: bool,
    line_has_const_global: bool,
    text: &str,
    rest: usize,
) -> HighlightKind {
    if word == "this" {
        return HighlightKind::LanguageSpecial;
    }
    if BUILTIN_TYPES.contains(&word) {
        return HighlightKind::Type;
    }
    if keywords.contains(&word) {
        return HighlightKind::Keyword;
    }
    if expect_type_name {
        return HighlightKind::Type;
    }
    if is_screaming_snake(word) {
        return HighlightKind::Constant;
    }
    let after = text.get(rest..).unwrap_or("");
    if after.starts_with('(') && !CALL_KEYWORD_BLOCKLIST.contains(&word) {
        return HighlightKind::Function;
    }
    if line_has_static && !after.starts_with('(') {
        return HighlightKind::StaticGlobal;
    }
    if line_has_const_global && !after.starts_with('(') {
        return HighlightKind::StaticGlobal;
    }
    if is_likely_parameter_or_local(text, word) {
        return HighlightKind::Parameter;
    }
    if is_member_access(text, word) {
        return HighlightKind::Property;
    }
    HighlightKind::Plain
}

fn is_likely_parameter_or_local(text: &str, word: &str) -> bool {
    if word.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
        return false;
    }
    let Some(pos) = text.find(word) else {
        return false;
    };
    let before = text[..pos].trim_end();
    BUILTIN_TYPES
        .iter()
        .any(|t| before.ends_with(t) || before.ends_with(&format!("{t}*")))
        || before.ends_with('_')
}

fn is_member_access(text: &str, word: &str) -> bool {
    let Some(pos) = text.find(word) else {
        return false;
    };
    if pos == 0 {
        return false;
    }
    let prev = text.as_bytes().get(pos.wrapping_sub(1)).copied();
    prev == Some(b'.') || text[..pos].ends_with("->")
}

fn try_match_ops(text: &str, i: usize, ops: &[&str], _kind: HighlightKind) -> Option<usize> {
    ops.iter()
        .find(|op| text[i..].starts_with(**op))
        .map(|op| op.len())
}

fn scan_string_spans(text: &str, i: usize, quote: char) -> (Vec<Span>, usize) {
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
            let esc_start = j;
            j += ch.len_utf8();
            if j < text.len() {
                j += text[j..].chars().next().unwrap().len_utf8();
            }
            spans.push(Span {
                start: esc_start,
                end: j,
                kind: HighlightKind::Escape,
            });
            continue;
        }
        if ch == quote {
            spans.push(Span {
                start: j,
                end: j + q_len,
                kind: HighlightKind::StringDelim,
            });
            let content_start = i + q_len;
            if j > content_start {
                spans.insert(
                    1,
                    Span {
                        start: content_start,
                        end: j,
                        kind: HighlightKind::String,
                    },
                );
            }
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

#[test]
fn param_rule_in_parens() {
    let spans = highlight_c_line("void f(int readonly, float number)");
    assert!(spans.iter().any(|s| s.kind == HighlightKind::Parameter));
}

#[test]
fn static_rule_only_next_ident() {
    let spans = highlight_c_line("static int32_t static_var = 3;");
    let names: Vec<_> = spans
        .iter()
        .filter(|s| s.kind == HighlightKind::StaticGlobal)
        .collect();
    assert_eq!(names.len(), 1);
}

#[test]
fn multiline_block_comment_middle_line() {
    let mut state = CfamilyHighlightState::default();
    let _ = highlight_c_line_with_state("/* start", &mut state);
    assert!(state.in_block_comment);
    let spans = highlight_c_line_with_state("   middle text", &mut state);
    assert!(spans.iter().all(|s| s.kind == HighlightKind::Comment));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_c_keywords_and_comment() {
        let spans = highlight_c_line("int main(void) { return 0; } // entry");
        assert!(spans.iter().any(|s| s.kind == HighlightKind::Type));
        assert!(spans.iter().any(|s| s.kind == HighlightKind::Comment));
    }

    #[test]
    fn highlights_cpp_class_and_string() {
        let spans = highlight_cpp_line("class Widget { std::string name = \"demo\"; };");
        assert!(spans.iter().any(|s| s.kind == HighlightKind::Keyword));
        assert!(spans.iter().any(|s| s.kind == HighlightKind::String));
    }

    #[test]
    fn highlights_preprocessor_as_macro_magenta() {
        let spans = highlight_c_line("#include <stdio.h>");
        assert!(spans.iter().any(|s| s.kind == HighlightKind::Macro));
        assert!(spans.iter().any(|s| s.kind == HighlightKind::String));
    }

    #[test]
    fn highlights_logical_operators_red() {
        let spans = highlight_c_line("if (a == b && c != d)");
        assert!(spans
            .iter()
            .any(|s| s.kind == HighlightKind::OperatorLogical));
    }

    #[test]
    fn highlights_define_macro_name() {
        let spans = highlight_c_line("#define KEEPALIVE_IDLE 1");
        assert!(
            spans
                .iter()
                .filter(|s| s.kind == HighlightKind::Macro)
                .count()
                >= 2
        );
    }
}

fn highlight_cfamily_code_for_mode(text: &str, cpp: bool) -> Vec<Span> {
    if cpp {
        let mut kws: Vec<&str> = C_KEYWORDS
            .iter()
            .copied()
            .chain(CPP_KEYWORDS.iter().copied())
            .collect();
        kws.sort_unstable();
        kws.dedup();
        highlight_cfamily_code(text, &kws)
    } else {
        highlight_cfamily_code(text, C_KEYWORDS)
    }
}
