use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Frame;

use crate::command::EditorMode;
use crate::document::Document;
use crate::search::SearchState;
use crate::syntax::Language;
use crate::theme::CcppTheme;

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
        _ => "OTHER",
    };
    let dirty = if doc.dirty { " +" } else { "" };
    let lang = match doc.lang {
        Language::Plain => "Plain",
        Language::C => "C",
        Language::Cpp => "C++",
        Language::Rust => "Rust",
    };
    let meta = format!(
        " Ln {}, Col {} | {} | {} | {}",
        doc.cursor.row + 1,
        doc.cursor.col + 1,
        doc.encoding.label(),
        doc.line_ending.label(),
        lang
    );
    let left = format!(" {}{}{} ", doc.display_name(), dirty, meta);
    let prompt_part = if prompt.is_empty() {
        status.to_string()
    } else {
        format!("{prompt}_")
    };
    let right = format!("{} | {}", mode_str, prompt_part);
    let pad = (area.width as usize).saturating_sub(left.len() + right.len());
    Paragraph::new(format!("{left}{}{right}", " ".repeat(pad)))
        .style(
            Style::default()
                .bg(CcppTheme::STATUS_BG)
                .fg(CcppTheme::STATUS_FG),
        )
        .render(area, frame.buffer_mut());
    let _ = search;
}
