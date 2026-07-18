//! Ratatui 渲染：布局、gutter、语法着色正文、选区与搜索高亮
//!
//! 坐标约定
//! - crate::cursor::Cursor 每行使用字符列
//! - crate::search::Match 每行使用字节偏移（与 str::find 一致）
//! - 选区 overlay 将字符列转换为字节边界（char_col_to_byte）

mod gutter;
mod layout;
mod status;
mod text;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Widget};
use ratatui::Frame;

use crate::command::EditorMode;
use crate::document::Document;
use crate::search::{Match, SearchState};
use crate::theme::CcppTheme;

pub use layout::{
    cursor_for_drag, drag_scroll_lines, is_in_editor_pane, screen_to_cursor, screen_to_gutter_row,
    visible_line_indices,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct EditorLayout {
    /// 行号 + 折叠标记列
    pub gutter: Rect,
    /// 正文区；screen_to_cursor 仅在此矩形内有效
    pub text: Rect,
}

#[derive(Debug, Clone)]
pub struct ViewState {
    /// 可见行窗口在 visible_line_map 中的起始下标
    pub scroll_row: usize,
    pub show_whitespace: bool,
    pub layout: EditorLayout,
    /// follow_cursor 为 false 时，视口保持不动直至再次移动光标
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

    /// 按 delta 可见行滚动视口（>0 向下，<0 向上）
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

    /// 将光标所在逻辑行滚入可见窗口（follow_cursor 为 true 时每帧调用）
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

pub struct EditorChrome<'a> {
    pub tab_labels: &'a [String],
    pub active: usize,
    pub mode: EditorMode,
    pub search: &'a SearchState,
    pub status: &'a str,
    pub prompt: &'a str,
    pub word_hits: &'a [Match],
}

pub struct RenderContext<'a> {
    pub doc: &'a mut Document,
    pub chrome: EditorChrome<'a>,
}

/// 一帧 TUI：标签栏 → gutter+正文 → 状态栏；更新 doc.view.layout
pub fn draw(frame: &mut Frame, ctx: RenderContext<'_>) {
    let doc = ctx.doc;
    let chrome = ctx.chrome;
    let area = frame.area();

    Block::default()
        .style(Style::default().bg(CcppTheme::EDITOR_BG))
        .render(area, frame.buffer_mut());

    let has_tabs = chrome.tab_labels.len() > 1;
    let chunks = if has_tabs {
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area)
    } else {
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area)
    };

    let (tab_area, editor_area, status_area) = if has_tabs {
        (Some(chunks[0]), chunks[1], chunks[2])
    } else {
        (None, chunks[0], chunks[1])
    };

    if let Some(tabs) = tab_area {
        gutter::render_tab_bar(frame, tabs, chrome.tab_labels, chrome.active);
    }

    let visible_map = visible_line_indices(&doc.buffer, &doc.folds);
    let visible_rows = editor_area.height as usize;
    if doc.view.follow_cursor {
        doc.view
            .ensure_cursor_visible(doc.cursor.row, visible_rows, &visible_map);
    }

    let line_num_width = gutter::line_number_width(doc.buffer.line_count());
    let inner = Layout::horizontal([Constraint::Length(line_num_width), Constraint::Min(1)])
        .split(editor_area);

    doc.view.layout = EditorLayout {
        gutter: inner[0],
        text: inner[1],
    };

    gutter::render_gutter(
        frame,
        inner[0],
        doc,
        &visible_map,
        doc.view.scroll_row,
        visible_rows,
    );
    text::render_text(
        frame,
        inner[1],
        doc,
        &visible_map,
        chrome.search,
        chrome.word_hits,
    );
    status::render_status(
        frame,
        status_area,
        doc,
        chrome.mode,
        chrome.search,
        chrome.status,
        chrome.prompt,
    );
}

#[cfg(test)]
mod tests {
    use super::ViewState;

    #[test]
    fn scroll_by_clamps_at_bounds() {
        let mut view = ViewState::new();
        view.scroll_by(5, 10, 50);
        assert_eq!(view.scroll_row, 5);
        assert!(!view.follow_cursor);
        view.scroll_by(-100, 10, 50);
        assert_eq!(view.scroll_row, 0);
        view.scroll_by(100, 10, 50);
        assert_eq!(view.scroll_row, 40);
    }
}
