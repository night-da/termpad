//! Plain 正文渲染（Commit 10 起叠加 syntax；Commit 09 起叠加 search 底色）

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::Frame;

use crate::document::Document;
use crate::search::{Match, SearchState};

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
        lines.push(Line::from(text));
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
