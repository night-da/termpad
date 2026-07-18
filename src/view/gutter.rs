//! 标签栏、行号 gutter，宽度 = 行号位数 + 折叠标记与边距（约 +5）

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span as TextSpan};
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Frame;

use crate::document::Document;
use crate::theme::CcppTheme;

/// 行号列宽：位数 + 折叠符与左右留白
pub fn line_number_width(line_count: usize) -> u16 {
    line_count.max(1).ilog10() as u16 + 5
}

/// 多标签时顶栏一行；active 标签反色
pub fn render_tab_bar(frame: &mut Frame, area: Rect, labels: &[String], active: usize) {
    let spans: Vec<TextSpan> = labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            if i == active {
                TextSpan::styled(
                    format!(" {label} "),
                    Style::default()
                        .fg(CcppTheme::TAB_ACTIVE_FG)
                        .bg(CcppTheme::TAB_ACTIVE_BG),
                )
            } else {
                TextSpan::styled(
                    format!(" {label} "),
                    Style::default()
                        .fg(CcppTheme::TAB_INACTIVE_FG)
                        .bg(CcppTheme::EDITOR_BG),
                )
            }
        })
        .collect();
    Paragraph::new(Line::from(spans)).render(area, frame.buffer_mut());
}

/// 可见窗口内的行号与折叠三角；当前光标行加粗
pub fn render_gutter(
    frame: &mut Frame,
    area: Rect,
    doc: &Document,
    visible_map: &[usize],
    scroll_row: usize,
    visible_rows: usize,
) {
    let buffer = &doc.buffer;
    let folds = &doc.folds;
    let cursor_row = doc.cursor.row;
    let mut lines = Vec::new();
    for i in 0..visible_rows {
        let Some(&logical) = visible_map.get(scroll_row + i) else {
            lines.push(Line::from(""));
            continue;
        };
        let fold_mark = if folds.is_folded_start(logical) {
            "▸ "
        } else {
            "  "
        };
        let num = format!(
            "{:>width$}",
            logical + 1,
            width = buffer.line_count().ilog10() as usize + 1
        );
        let mut style = Style::default().fg(CcppTheme::LINE_NUMBER);
        if logical == cursor_row {
            style = style
                .add_modifier(Modifier::BOLD)
                .fg(CcppTheme::LINE_NUMBER_ACTIVE);
        }
        lines.push(Line::from(vec![
            TextSpan::styled(fold_mark, Style::default().fg(CcppTheme::GUTTER_MARK)),
            TextSpan::styled(num, style),
            TextSpan::styled(" ", Style::default().bg(CcppTheme::EDITOR_BG)),
        ]));
    }
    Paragraph::new(lines).render(area, frame.buffer_mut());
}
