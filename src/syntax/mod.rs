//! 按行语法高亮（C/C++/Rust/Markdown），配色来自 ccpp_theme
//!
//! 对外入口 highlight_line；语言由 detect_language 按扩展名推断，均为单遍启发式，非编译器级语义

mod cfamily;
pub use cfamily::{
    advance_block_comment_state, highlight_c_line_with_state, highlight_cpp_line_with_state,
    CfamilyHighlightState,
};
mod markdown;
mod merge;
mod rust;
mod scan;

use ratatui::style::{Modifier, Style};

use crate::theme::CcppTheme;

/// 由 detect_language 按扩展名推断；无路径时为 Plain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Plain,
    C,
    Cpp,
    Rust,
    Markdown,
}

/// 语法分类（映射到 CcppTheme 颜色）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightKind {
    Plain,
    Comment,
    Keyword,
    Type,
    Function,
    Macro,
    Constant,
    String,
    StringDelim,
    Number,
    Parameter,
    StaticGlobal,
    Property,
    OperatorLogical,
    Operator,
    Punctuation,
    Escape,
    Label,
    LanguageSpecial,
    Heading,
    Bold,
    Italic,
    Code,
    Link,
    LinkUrl,
    ListMark,
    Quote,
    HorizontalRule,
}

/// 语法片段；start/end 均为行内 UTF-8 字节边界
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub kind: HighlightKind,
}

pub fn style_for(kind: HighlightKind) -> Style {
    match kind {
        HighlightKind::Plain => Style::default().fg(CcppTheme::PLAIN),
        HighlightKind::Comment => Style::default().fg(CcppTheme::COMMENT),
        HighlightKind::Keyword => Style::default().fg(CcppTheme::KEYWORD),
        HighlightKind::Type => Style::default().fg(CcppTheme::TYPE),
        HighlightKind::Function => Style::default().fg(CcppTheme::FUNCTION),
        HighlightKind::Macro => Style::default()
            .fg(CcppTheme::MACRO)
            .add_modifier(Modifier::BOLD),
        HighlightKind::Constant => Style::default().fg(CcppTheme::CONSTANT),
        HighlightKind::String => Style::default().fg(CcppTheme::STRING),
        HighlightKind::StringDelim => Style::default().fg(CcppTheme::STRING_DELIM),
        HighlightKind::Number => Style::default().fg(CcppTheme::NUMBER),
        HighlightKind::Parameter => Style::default()
            .fg(CcppTheme::PARAMETER)
            .add_modifier(Modifier::ITALIC),
        HighlightKind::StaticGlobal => Style::default()
            .fg(CcppTheme::STATIC_GLOBAL)
            .add_modifier(Modifier::BOLD),
        HighlightKind::Property => Style::default().fg(CcppTheme::PROPERTY),
        HighlightKind::OperatorLogical => Style::default().fg(CcppTheme::OPERATOR_LOGICAL),
        HighlightKind::Operator => Style::default().fg(CcppTheme::OPERATOR),
        HighlightKind::Punctuation => Style::default().fg(CcppTheme::PUNCTUATION),
        HighlightKind::Escape => Style::default().fg(CcppTheme::ESCAPE),
        HighlightKind::Label => Style::default()
            .fg(CcppTheme::LABEL)
            .add_modifier(Modifier::UNDERLINED),
        HighlightKind::LanguageSpecial => Style::default()
            .fg(CcppTheme::LANGUAGE_SPECIAL)
            .add_modifier(Modifier::ITALIC),
        HighlightKind::Heading => Style::default()
            .fg(CcppTheme::HEADING)
            .add_modifier(Modifier::BOLD),
        HighlightKind::Bold => Style::default().fg(CcppTheme::BOLD),
        HighlightKind::Italic => Style::default()
            .fg(CcppTheme::ITALIC)
            .add_modifier(Modifier::ITALIC),
        HighlightKind::Code => Style::default().fg(CcppTheme::CODE),
        HighlightKind::Link => Style::default().fg(CcppTheme::LINK),
        HighlightKind::LinkUrl => Style::default().fg(CcppTheme::LINK_URL),
        HighlightKind::ListMark => Style::default().fg(CcppTheme::LIST_MARK),
        HighlightKind::Quote => Style::default()
            .fg(CcppTheme::QUOTE)
            .add_modifier(Modifier::ITALIC),
        HighlightKind::HorizontalRule => Style::default().fg(CcppTheme::HORIZONTAL_RULE),
    }
}

/// 按语言对一行文本着色；Plain 整行无高亮
pub fn highlight_line(line: &str, lang: Language) -> Vec<Span> {
    match lang {
        Language::Plain => vec![Span {
            start: 0,
            end: line.len(),
            kind: HighlightKind::Plain,
        }],
        Language::C => cfamily::highlight_c_line(line),
        Language::Cpp => cfamily::highlight_cpp_line(line),
        Language::Rust => rust::highlight_rust_line(line),
        Language::Markdown => markdown::highlight_markdown_line(line),
    }
}

/// 由路径扩展名推断语言；无路径或无扩展名时为 Plain
pub fn detect_language(path: Option<&std::path::Path>) -> Language {
    path.and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .map(|ext| match ext.to_ascii_lowercase().as_str() {
            "c" | "h" => Language::C,
            "cpp" | "cxx" | "cc" | "hpp" | "hxx" => Language::Cpp,
            "rs" => Language::Rust,
            "md" | "markdown" => Language::Markdown,
            _ => Language::Plain,
        })
        .unwrap_or(Language::Plain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_markdown_ext() {
        assert_eq!(
            detect_language(Some(std::path::Path::new("README.md"))),
            Language::Markdown
        );
    }

    #[test]
    fn detects_c_and_cpp_ext() {
        assert_eq!(
            detect_language(Some(std::path::Path::new("main.c"))),
            Language::C
        );
        assert_eq!(
            detect_language(Some(std::path::Path::new("app.cpp"))),
            Language::Cpp
        );
    }

    #[test]
    fn highlights_ccpp_c_sample_without_panic() {
        let text = include_str!("../../demos/demo.c");
        for line in text.lines() {
            let _ = highlight_line(line, Language::C);
        }
    }

    #[test]
    fn highlights_ccpp_cpp_sample_without_panic() {
        let text = include_str!("../../demos/demo.cpp");
        for line in text.lines() {
            let _ = highlight_line(line, Language::Cpp);
        }
    }

    #[test]
    fn highlights_rust_demo_without_panic() {
        let text = include_str!("../../demos/demo.rs");
        for line in text.lines() {
            let _ = highlight_line(line, Language::Rust);
        }
    }
}
