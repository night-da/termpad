//! 状态栏（模式、元数据、搜索信息）

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Frame;

use crate::command::EditorMode;
use crate::document::Document;
use crate::search::SearchState;
use crate::syntax::Language;
use crate::theme::CcppTheme;

/// 底部一行：文件名、行列、编码、模式、prompt
pub fn render_status(
    frame: &mut Frame,
    area: Rect,
    doc: &Document,
    mode: EditorMode,
    search: &SearchState,
    status: &str,
    prompt: &str,
) {
    let mode_str = match mode {
        EditorMode::Normal => "NORMAL",
        EditorMode::Insert => "INSERT",
        EditorMode::ColumnInsert => "COL-INS",
        EditorMode::SearchForward => "FIND",
        EditorMode::SearchBackward => "FIND?",
        EditorMode::ReplaceInput => "REPL-IN",
        EditorMode::ReplaceConfirm => "REPL",
        EditorMode::GotoLine => "GOTO",
        EditorMode::OpenPath => "OPEN",
        EditorMode::QuitConfirm => "QUIT?",
        EditorMode::CloseTabConfirm => "CLOSE?",
    };
    let dirty = if doc.dirty { " +" } else { "" };
    let ro = "";
    let lang = match doc.lang {
        Language::Plain => "Plain",
        Language::C => "C",
        Language::Cpp => "C++",
        Language::Rust => "Rust",
        Language::Markdown => "Markdown",
    };
    let meta = format!(
        " Ln {}, Col {} | {} | {} | {}{}",
        doc.cursor.row + 1,
        doc.cursor.col + 1,
        doc.encoding.label(),
        doc.line_ending.label(),
        lang,
        ro
    );
    let search_info = if search.query.is_empty() {
        String::new()
    } else {
        format!(
            " | /{} [{}] {}/{}",
            search.query,
            search.options_label(),
            search.current.map(|i| i + 1).unwrap_or(0),
            search.matches.len()
        )
    };
    let ws = if doc.view.show_whitespace {
        " | WS"
    } else {
        ""
    };
    let left = format!(
        " {}{}{}{}{} ",
        doc.display_name(),
        dirty,
        meta,
        search_info,
        ws
    );
    let prompt_part = if prompt.is_empty() {
        status.to_string()
    } else {
        format!("{prompt}_")
    };
    let right = format!("{} | {}", mode_str, prompt_part);
    let width = area.width as usize;
    let pad = width.saturating_sub(left.len() + right.len());
    Paragraph::new(format!("{left}{}{right}", " ".repeat(pad)))
        .style(
            Style::default()
                .bg(CcppTheme::STATUS_BG)
                .fg(CcppTheme::STATUS_FG),
        )
        .render(area, frame.buffer_mut());
}
