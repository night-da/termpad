//! 正文渲染：语法着色 + 搜索/选区底色叠加 + 硬件光标
//!
//! 底色优先级（后者覆盖前者）：当前搜索匹配 → 其他匹配 → 选区 → 纯语法

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span as TextSpan};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::Frame;

use crate::document::Document;
use crate::search::{Match, SearchState};
use crate::selection::Selection;
use crate::syntax::{
    advance_block_comment_state, highlight_c_line_with_state, highlight_cpp_line_with_state,
    highlight_line, style_for, CfamilyHighlightState, HighlightKind, Language,
};
use crate::theme::CcppTheme;

struct LineRenderContext<'a> {
    row: usize,
    search: &'a SearchState,
    word_hits: &'a [Match],
    show_ws: bool,
    selection: &'a Selection,
    cursor: crate::cursor::Cursor,
    line_len: usize,
    syntax: &'a [crate::syntax::Span],
}

pub fn render_text(
    frame: &mut Frame,
    area: Rect,
    doc: &Document,
    visible_map: &[usize],
    search: &SearchState,
    word_hits: &[Match],
) {
    let visible_rows = area.height as usize;
    let mut cf_state = CfamilyHighlightState::default();
    if matches!(doc.lang, Language::C | Language::Cpp) {
        // C/C++ 块注释跨行：从文件头扫到视口首行，否则块注释状态错误
        if let Some(&first_visible) = visible_map.get(doc.view.scroll_row) {
            for row in 0..first_visible {
                if let Some(line) = doc.buffer.line(row) {
                    advance_block_comment_state(&line, &mut cf_state);
                }
            }
        }
    }
    let mut lines = Vec::new();
    for i in 0..visible_rows {
        let Some(&row) = visible_map.get(doc.view.scroll_row + i) else {
            lines.push(Line::from(""));
            continue;
        };
        let text = doc.buffer.line(row).unwrap_or_default();
        let line_len = text.chars().count();
        let syntax = match doc.lang {
            Language::C => highlight_c_line_with_state(&text, &mut cf_state),
            Language::Cpp => highlight_cpp_line_with_state(&text, &mut cf_state),
            lang => highlight_line(&text, lang),
        };
        let line_ctx = LineRenderContext {
            row,
            search,
            word_hits,
            show_ws: doc.view.show_whitespace,
            selection: &doc.selection,
            cursor: doc.cursor,
            line_len,
            syntax: &syntax,
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
    let sel_cols = ctx
        .selection
        .cols_on_line(ctx.row, ctx.cursor, ctx.line_len);

    if raw.is_empty() {
        if sel_cols.is_some() {
            return vec![TextSpan::styled(
                " ",
                Style::default().bg(CcppTheme::SELECTION_BG),
            )];
        }
        return vec![TextSpan::raw("")];
    }

    let syntax = ctx.syntax;
    let mut breaks = std::collections::BTreeSet::new();
    breaks.insert(0);
    breaks.insert(raw.len());
    for hs in syntax {
        breaks.insert(hs.start.min(raw.len()));
        breaks.insert(hs.end.min(raw.len()));
    }
    for m in ctx.search.matches.iter().filter(|m| m.row == ctx.row) {
        breaks.insert(m.col.min(raw.len()));
        breaks.insert((m.col + m.len).min(raw.len()));
    }
    for m in ctx.word_hits.iter().filter(|m| m.row == ctx.row) {
        breaks.insert(m.col.min(raw.len()));
        breaks.insert((m.col + m.len).min(raw.len()));
    }
    if let Some((sel_start, sel_end)) = sel_cols {
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
        let slice_raw = safe_byte_range(raw, start, end);
        if slice_raw.is_empty() {
            continue;
        }
        let text = if ctx.show_ws {
            visualize_whitespace(slice_raw)
        } else {
            slice_raw.to_string()
        };
        let style = segment_style(start, end, raw, syntax, ctx, sel_cols);
        spans.push(TextSpan::styled(text, style));
    }
    spans
}

fn char_col_to_byte(line: &str, col: usize) -> usize {
    line.char_indices()
        .nth(col)
        .map(|(i, _)| i)
        .unwrap_or(line.len())
}

fn segment_style(
    start: usize,
    end: usize,
    line: &str,
    syntax: &[crate::syntax::Span],
    ctx: &LineRenderContext<'_>,
    sel_cols: Option<(usize, usize)>,
) -> Style {
    let mid = start + (end - start) / 2;
    let kind = syntax
        .iter()
        .find(|hs| mid >= hs.start && mid < hs.end)
        .map(|hs| hs.kind)
        .unwrap_or(HighlightKind::Plain);
    let style = style_for(kind);

    if segment_in_match(start, end, ctx.row, ctx.search.current_match()) {
        return style.bg(CcppTheme::FIND_CURRENT_BG);
    }
    if ctx
        .search
        .matches
        .iter()
        .any(|m| m.row == ctx.row && ranges_overlap(start, end, m.col, m.col + m.len))
    {
        return style.bg(CcppTheme::FIND_OTHER_BG);
    }
    if segment_in_selection(start, end, line, sel_cols) {
        return style.bg(CcppTheme::SELECTION_BG);
    }
    if ctx
        .word_hits
        .iter()
        .any(|m| m.row == ctx.row && ranges_overlap(start, end, m.col, m.col + m.len))
    {
        return style.bg(CcppTheme::WORD_HIGHLIGHT_BG);
    }
    style.bg(CcppTheme::EDITOR_BG)
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

fn visualize_whitespace(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => '·',
            '\t' => '→',
            _ => c,
        })
        .collect()
}
