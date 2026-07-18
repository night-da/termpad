//! Plain 正文渲染 + 选区/搜索底色（Commit 10 起叠加 syntax）

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span as TextSpan};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::Frame;

use crate::document::Document;
use crate::search::{Match, SearchState};
use crate::theme::CcppTheme;

struct LineRenderContext<'a> {
    row: usize,
    search: &'a SearchState,
    sel_cols: Option<(usize, usize)>,
}

pub fn render_text(
    frame: &mut Frame,
    area: Rect,
    doc: &Document,
    visible_map: &[usize],
    search: &SearchState,
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
        let line_ctx = LineRenderContext {
            row,
            search,
            sel_cols: doc.selection.cols_on_line(row, doc.cursor, line_len),
        };
        let mut spans = build_line_spans(&text, &line_ctx);
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

fn build_line_spans(raw: &str, ctx: &LineRenderContext<'_>) -> Vec<TextSpan<'static>> {
    if raw.is_empty() {
        if ctx.sel_cols.is_some() {
            return vec![TextSpan::styled(
                " ",
                Style::default().bg(CcppTheme::SELECTION_BG),
            )];
        }
        return vec![TextSpan::raw("")];
    }

    let mut breaks = std::collections::BTreeSet::new();
    breaks.insert(0);
    breaks.insert(raw.len());
    for m in ctx.search.matches.iter().filter(|m| m.row == ctx.row) {
        breaks.insert(m.col.min(raw.len()));
        breaks.insert((m.col + m.len).min(raw.len()));
    }
    if let Some((sel_start, sel_end)) = ctx.sel_cols {
        breaks.insert(char_col_to_byte(raw, sel_start));
        breaks.insert(char_col_to_byte(raw, sel_end));
    }

    let points: Vec<usize> = breaks.into_iter().collect();
    let mut spans = Vec::new();
    for window in points.windows(2) {
        let start = window[0];
        let end = window[1];
        if start >= end {
            continue;
        }
        let slice = safe_byte_range(raw, start, end);
        if slice.is_empty() {
            continue;
        }
        let style = segment_style(start, end, raw, ctx);
        spans.push(TextSpan::styled(
            slice.to_string(),
            Style::default()
                .fg(CcppTheme::PLAIN)
                .bg(style.bg.unwrap_or(CcppTheme::EDITOR_BG)),
        ));
    }
    spans
}

fn segment_style(start: usize, end: usize, line: &str, ctx: &LineRenderContext<'_>) -> Style {
    if segment_in_match(start, end, ctx.row, ctx.search.current_match()) {
        return Style::default().bg(CcppTheme::FIND_CURRENT_BG);
    }
    if ctx
        .search
        .matches
        .iter()
        .any(|m| m.row == ctx.row && ranges_overlap(start, end, m.col, m.col + m.len))
    {
        return Style::default().bg(CcppTheme::FIND_OTHER_BG);
    }
    if segment_in_selection(start, end, line, ctx.sel_cols) {
        return Style::default().bg(CcppTheme::SELECTION_BG);
    }
    Style::default().bg(CcppTheme::EDITOR_BG)
}

fn char_col_to_byte(line: &str, col: usize) -> usize {
    line.char_indices()
        .nth(col)
        .map(|(i, _)| i)
        .unwrap_or(line.len())
}

fn segment_in_match(start: usize, end: usize, row: usize, m: Option<&Match>) -> bool {
    m.is_some_and(|m| m.row == row && ranges_overlap(start, end, m.col, m.col + m.len))
}

fn segment_in_selection(
    byte_start: usize,
    byte_end: usize,
    line: &str,
    sel_cols: Option<(usize, usize)>,
) -> bool {
    let Some((sel_start, sel_end)) = sel_cols else {
        return false;
    };
    if sel_start >= sel_end {
        return false;
    }
    let seg_start = line[..byte_start.min(line.len())].chars().count();
    let seg_end = line[..byte_end.min(line.len())].chars().count();
    seg_start < sel_end && sel_start < seg_end
}

fn ranges_overlap(a_start: usize, a_end: usize, b_start: usize, b_end: usize) -> bool {
    a_start < b_end && b_start < a_end
}

fn safe_byte_range(text: &str, start: usize, end: usize) -> &str {
    let start = floor_char_boundary(text, start);
    let end = ceil_char_boundary(text, end.max(start));
    if start >= end {
        return "";
    }
    &text[start..end]
}

fn floor_char_boundary(text: &str, pos: usize) -> usize {
    let mut pos = pos.min(text.len());
    while pos > 0 && !text.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

fn ceil_char_boundary(text: &str, pos: usize) -> usize {
    let mut pos = pos.min(text.len());
    while pos < text.len() && !text.is_char_boundary(pos) {
        pos += 1;
    }
    pos
}
