//! 视口状态 stub：Commit 05 引入 ratatui 后升级 EditorLayout 为 Rect

#[derive(Debug, Clone, Copy, Default)]
pub struct EditorLayout {}

#[derive(Debug, Clone)]
pub struct ViewState {
    pub scroll_row: usize,
    pub show_whitespace: bool,
    pub layout: EditorLayout,
    pub follow_cursor: bool,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            scroll_row: 0,
            show_whitespace: false,
            layout: EditorLayout::default(),
            follow_cursor: true,
        }
    }

    pub fn scroll_by(&mut self, delta: i32, visible_rows: usize, total_visible_lines: usize) {
        if visible_rows == 0 {
            return;
        }
        self.follow_cursor = false;
        if total_visible_lines <= visible_rows {
            self.scroll_row = 0;
            return;
        }
        let max_scroll = total_visible_lines - visible_rows;
        if delta < 0 {
            self.scroll_row = self.scroll_row.saturating_sub((-delta) as usize);
        } else {
            self.scroll_row = (self.scroll_row + delta as usize).min(max_scroll);
        }
    }

    pub fn toggle_whitespace(&mut self) {
        self.show_whitespace = !self.show_whitespace;
    }

    pub fn ensure_cursor_visible(
        &mut self,
        cursor_row: usize,
        visible_rows: usize,
        visible_line_map: &[usize],
    ) {
        let Some(pos) = visible_line_map.iter().position(|&r| r == cursor_row) else {
            return;
        };
        if pos < self.scroll_row {
            self.scroll_row = pos;
        } else if pos >= self.scroll_row + visible_rows {
            self.scroll_row = pos.saturating_sub(visible_rows.saturating_sub(1));
        }
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new()
    }
}
