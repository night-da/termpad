//! 视口映射：可见行、屏幕坐标 → 光标
//!
//! visible_line_indices 已排除折叠隐藏行

use crate::buffer::GapBuffer;
use crate::document::Document;
use crate::fold::FoldState;
use ratatui::layout::Rect;

/// 未折叠的逻辑行号列表（渲染与 scroll 的下标空间）
pub fn visible_line_indices(buffer: &GapBuffer, folds: &FoldState) -> Vec<usize> {
    (0..buffer.line_count())
        .filter(|&r| !folds.is_hidden(r))
        .collect()
}

/// (x, y) 落在行号 gutter 或正文区时为 true
pub fn is_in_editor_pane(doc: &Document, x: u16, y: u16) -> bool {
    let pos = ratatui::layout::Position { x, y };
    doc.view.layout.text.contains(pos) || doc.view.layout.gutter.contains(pos)
}

/// 屏幕坐标 → buffer 光标；正文区外返回 None
pub fn screen_to_cursor(doc: &Document, x: u16, y: u16) -> Option<crate::cursor::Cursor> {
    let area = doc.view.layout.text;
    if area.width == 0 || area.height == 0 {
        return None;
    }
    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }
    let display_row = (y - area.y) as usize;
    let visible_map = visible_line_indices(&doc.buffer, &doc.folds);
    let logical_row = *visible_map.get(doc.view.scroll_row + display_row)?;
    let rel_x = (x - area.x) as usize;
    let line = doc.buffer.line(logical_row)?;
    let line_len = line.chars().count();
    let col = rel_x.min(line_len);
    Some(crate::cursor::Cursor {
        row: logical_row,
        col,
    })
}

/// 屏幕坐标 → gutter 行号（不含列）
pub fn screen_to_gutter_row(doc: &Document, x: u16, y: u16) -> Option<usize> {
    let area = doc.view.layout.gutter;
    if area.width == 0 || area.height == 0 {
        return None;
    }
    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }
    let display_row = (y - area.y) as usize;
    let visible_map = visible_line_indices(&doc.buffer, &doc.folds);
    visible_map.get(doc.view.scroll_row + display_row).copied()
}

/// 离文本区越远，单次滚动行数越多（2–10 行/次）
fn drag_scroll_edge_speed(distance: u16) -> i32 {
    let d = i32::from(distance.max(1));
    (d + 1).clamp(2, 10)
}

fn drag_scroll_outside_speed(distance: u16) -> i32 {
    i32::from(distance.max(1))
}

/// 拖拽选区时的滚动行数，负值向上、正值向下
pub fn drag_scroll_lines(
    y: u16,
    area: Rect,
    scroll_row: usize,
    visible_rows: usize,
    total_visible: usize,
) -> i32 {
    if area.height == 0 || visible_rows == 0 || total_visible <= visible_rows {
        return 0;
    }
    let max_scroll = total_visible - visible_rows;
    let top = area.y;
    let bottom = area.y + area.height;

    let mut delta = if y < top {
        -drag_scroll_outside_speed(top - y)
    } else if y >= bottom {
        drag_scroll_outside_speed(y - bottom + 1)
    } else {
        let rel = y - top;
        const EDGE_ROWS: u16 = 1;
        if rel < EDGE_ROWS && scroll_row > 0 {
            -i32::from(EDGE_ROWS - rel)
        } else if rel >= area.height.saturating_sub(EDGE_ROWS) && scroll_row < max_scroll {
            drag_scroll_edge_speed(rel - area.height.saturating_sub(EDGE_ROWS) + 1)
        } else {
            0
        }
    };

    if (delta < 0 && scroll_row == 0) || (delta > 0 && scroll_row >= max_scroll) {
        delta = 0;
    }
    delta
}

/// 拖拽时根据坐标映射光标；y 超出正文区时自动滚动
pub fn cursor_for_drag(doc: &mut Document, x: u16, y: u16) -> Option<crate::cursor::Cursor> {
    let area = doc.view.layout.text;
    if area.width == 0 || area.height == 0 {
        return None;
    }
    let visible_map = visible_line_indices(&doc.buffer, &doc.folds);
    let visible_rows = area.height as usize;
    let total = visible_map.len();
    if total == 0 {
        return None;
    }

    let delta = drag_scroll_lines(y, area, doc.view.scroll_row, visible_rows, total);
    if delta != 0 {
        doc.view.scroll_by(delta, visible_rows, total);
    }

    let display_row = if y < area.y {
        0
    } else if y >= area.y + area.height {
        visible_rows.saturating_sub(1)
    } else {
        (y - area.y) as usize
    };

    let logical_row = *visible_map.get(doc.view.scroll_row + display_row)?;
    let line_len = doc.buffer.line_len(logical_row);
    let col = if x < area.x {
        0
    } else if x >= area.x + area.width {
        line_len
    } else {
        ((x - area.x) as usize).min(line_len)
    };

    Some(crate::cursor::Cursor {
        row: logical_row,
        col,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    fn sample_doc(line_count: usize, visible_height: u16) -> crate::document::Document {
        let mut doc = crate::document::Document::new_empty(None);
        let mut text = String::new();
        for i in 0..line_count {
            if i > 0 {
                text.push('\n');
            }
            text.push_str(&format!("line {i}"));
        }
        doc.buffer = crate::buffer::GapBuffer::from_str(&text);
        doc.view.layout.text = Rect {
            x: 10,
            y: 5,
            width: 20,
            height: visible_height,
        };
        doc
    }

    #[test]
    fn drag_at_top_visible_row_scrolls_up() {
        let mut doc = sample_doc(10, 3);
        doc.view.scroll_row = 2;
        let area = doc.view.layout.text;
        let cursor = cursor_for_drag(&mut doc, 12, area.y).unwrap();
        assert_eq!(doc.view.scroll_row, 1);
        assert_eq!(cursor.row, 1);
    }

    #[test]
    fn drag_scroll_speed_increases_with_distance() {
        let area = Rect {
            x: 0,
            y: 5,
            width: 20,
            height: 10,
        };
        assert_eq!(drag_scroll_lines(4, area, 5, 10, 50), -1);
        assert_eq!(drag_scroll_lines(0, area, 5, 10, 50), -5);
        assert_eq!(drag_scroll_lines(16, area, 5, 10, 50), 2);
    }
}
