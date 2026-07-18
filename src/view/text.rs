//! Plain 正文渲染 + 选区底色（Commit 09 起叠加 search；Commit 10 起叠加 syntax）

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span as TextSpan};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::Frame;

use crate::document::Document;
use crate::search::{Match, SearchState};
use crate::theme::CcppTheme;

pub fn render_text(
    frame: &mut Frame,
    area: Rect,
    doc: &Document,
    visible_map: &[usize],
    _search: &SearchState,
    _word_hits: &[Match],
) {
    let visible_rows = area.height as usize;
    let mut lines = Vec::new();
    for i in 0..visible_rows {
        let Some(&row) = visible_map.get(doc.view.scroll_row + i) else {
            lines.push(Line::from(""));
            continue;
        };
        let text = doc.buffer.line(row).unwrap_or_default();
        let line_len = text.chars().count();
        let sel_cols = doc.selection.cols_on_line(row, doc.cursor, line_len);
        let mut spans = build_line_spans(&text, sel_cols);
        if spans.is_empty() {
            spans.push(TextSpan::raw(""));
        }
        lines.push(Line::from(spans));
    }
    let block = Block::default();
    let inner = block.inner(area);
    Paragraph::new(lines).render(inner, frame.buffer_mut());

    let Some(map_pos) = visible_map.iter().position(|&r| r == doc.cursor.row) else {
        return;
    };
    if map_pos < doc.view.scroll_row || map_pos >= doc.view.scroll_row + visible_rows {
        return;
    }
    let cursor_display_row = map_pos - doc.view.scroll_row;
    let line_len = doc.buffer.line_len(doc.cursor.row);
    let col = doc.cursor.col.min(line_len) as u16;
    let cursor_x = inner.x + col.min(inner.width.saturating_sub(1));
    let cursor_y = inner.y + cursor_display_row as u16;
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn build_line_spans(raw: &str, sel_cols: Option<(usize, usize)>) -> Vec<TextSpan<'static>> {
    if raw.is_empty() {
        if sel_cols.is_some() {
            return vec![TextSpan::styled(
                " ",
                Style::default().bg(CcppTheme::SELECTION_BG),
            )];
        }
        return vec![TextSpan::raw("")];
    }

    let Some((sel_start, sel_end)) = sel_cols else {
        return vec![TextSpan::styled(
            raw.to_string(),
            Style::default()
                .fg(CcppTheme::PLAIN)
                .bg(CcppTheme::EDITOR_BG),
        )];
    };

    if sel_start >= sel_end {
        return vec![TextSpan::styled(
            raw.to_string(),
            Style::default()
                .fg(CcppTheme::PLAIN)
                .bg(CcppTheme::EDITOR_BG),
        )];
    }

    let mut spans = Vec::new();
    let chars: Vec<(usize, char)> = raw.char_indices().collect();
    let mut byte_start = 0;
    let mut col = 0;
    let mut in_sel = false;

    for (byte_idx, _ch) in chars {
        let next_in_sel = col >= sel_start && col < sel_end;
        if col > 0 && next_in_sel != in_sel {
            let slice = &raw[byte_start..byte_idx];
            if !slice.is_empty() {
                let style = if in_sel {
                    Style::default()
                        .fg(CcppTheme::PLAIN)
                        .bg(CcppTheme::SELECTION_BG)
                } else {
                    Style::default()
                        .fg(CcppTheme::PLAIN)
                        .bg(CcppTheme::EDITOR_BG)
                };
                spans.push(TextSpan::styled(slice.to_string(), style));
            }
            byte_start = byte_idx;
            in_sel = next_in_sel;
        } else if col == 0 {
            in_sel = next_in_sel;
        }
        col += 1;
    }

    let tail = &raw[byte_start..];
    if !tail.is_empty() {
        let style = if in_sel {
            Style::default()
                .fg(CcppTheme::PLAIN)
                .bg(CcppTheme::SELECTION_BG)
        } else {
            Style::default()
                .fg(CcppTheme::PLAIN)
                .bg(CcppTheme::EDITOR_BG)
        };
        spans.push(TextSpan::styled(tail.to_string(), style));
    }

    spans
}
