mod merge;
mod rust;
mod scan;

use ratatui::style::{Modifier, Style};

use crate::theme::CcppTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Plain,
    Rust,
}

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
}

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
    }
}

pub fn highlight_line(line: &str, lang: Language) -> Vec<Span> {
    match lang {
        Language::Plain => vec![Span {
            start: 0,
            end: line.len(),
            kind: HighlightKind::Plain,
        }],
        Language::Rust => rust::highlight_rust_line(line),
    }
}

pub fn detect_language(path: Option<&std::path::Path>) -> Language {
    path.and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .map(|ext| match ext.to_ascii_lowercase().as_str() {
            "rs" => Language::Rust,
            _ => Language::Plain,
        })
        .unwrap_or(Language::Plain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rust_ext() {
        assert_eq!(
            detect_language(Some(std::path::Path::new("main.rs"))),
            Language::Rust
        );
    }

    #[test]
    fn highlights_rust_demo_without_panic() {
        let text = include_str!("../../demos/demo.rs");
        for line in text.lines() {
            let _ = highlight_line(line, Language::Rust);
        }
    }
}
