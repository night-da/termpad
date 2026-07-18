//! 鼠标：点击、拖拽选区、滚轮
//!
//! 左键正文区：定位光标并开始选区；gutter：整行定位
//! 拖拽超出正文边缘时由 layout::cursor_for_drag 自动滚动

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::command::{Command, EditorMode};
use crate::view::{
    drag_scroll_lines, is_in_editor_pane, screen_to_cursor, screen_to_gutter_row,
    visible_line_indices,
};

use super::App;

impl App {
    /// 将屏幕坐标转为 Command 并交给 handle；非编辑模式忽略
    pub(super) fn handle_mouse(&mut self, mouse: MouseEvent) -> bool {
        if !matches!(
            self.mode,
            EditorMode::Normal | EditorMode::Insert | EditorMode::ColumnInsert
        ) {
            return false;
        }
        let mut changed = false;
        let cmd = match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => Some(Command::MouseDown {
                row: mouse.row,
                col: mouse.column,
            }),
            MouseEventKind::Drag(MouseButton::Left) if self.mouse_selecting => {
                Some(Command::MouseDrag {
                    row: mouse.row,
                    col: mouse.column,
                })
            }
            MouseEventKind::Up(MouseButton::Left) => Some(Command::MouseUp),
            MouseEventKind::ScrollUp
                if is_in_editor_pane(&self.documents[self.active], mouse.column, mouse.row) =>
            {
                Some(Command::ScrollUp { lines: 3 })
            }
            MouseEventKind::ScrollDown
                if is_in_editor_pane(&self.documents[self.active], mouse.column, mouse.row) =>
            {
                Some(Command::ScrollDown { lines: 3 })
            }
            _ => None,
        };
        if let Some(cmd) = cmd {
            self.handle(cmd);
            changed = true;
        }
        changed
    }

    pub(super) fn handle_mouse_down(&mut self, row: u16, col: u16) {
        let i = self.active;
        let cursor_opt = screen_to_cursor(&self.documents[i], col, row);
        let gutter_opt = screen_to_gutter_row(&self.documents[i], col, row);

        if let Some(cursor) = cursor_opt {
            self.mouse_selecting = true;
            self.selection_drag_start = Some(cursor);
            let doc = &mut self.documents[i];
            doc.view.follow_cursor = true;
            doc.selection.clear();
            doc.cursor = cursor;
        } else if let Some(line_row) = gutter_opt {
            self.mouse_selecting = false;
            self.selection_drag_start = None;
            let doc = &mut self.documents[i];
            doc.view.follow_cursor = true;
            doc.selection.clear();
            doc.cursor.row = line_row;
            doc.cursor.col = 0;
            doc.cursor.clamp(&doc.buffer);
        }
    }

    /// 扩展选区时更新 head；滚动发生后也视为选区变化
    pub(super) fn handle_mouse_drag(&mut self, row: u16, col: u16) -> bool {
        self.mouse_drag_pos = Some((col, row));
        let Some(start) = self.selection_drag_start else {
            return false;
        };
        let i = self.active;
        let doc = &mut self.documents[i];
        let scroll_before = doc.view.scroll_row;
        let Some(cursor) = crate::view::cursor_for_drag(doc, col, row) else {
            return false;
        };
        let scrolled = doc.view.scroll_row != scroll_before;
        if start != cursor || scrolled {
            doc.view.follow_cursor = false;
            doc.selection.begin(start);
            doc.cursor = cursor;
            return true;
        }
        false
    }

    pub(super) fn drag_autoscroll_active(&self) -> bool {
        let Some((_, row)) = self.mouse_drag_pos else {
            return false;
        };
        if !self.mouse_selecting {
            return false;
        }
        let doc = self.active_doc();
        let area = doc.view.layout.text;
        let visible_rows = area.height as usize;
        if visible_rows == 0 {
            return false;
        }
        let total = visible_line_indices(&doc.buffer, &doc.folds).len();
        drag_scroll_lines(row, area, doc.view.scroll_row, visible_rows, total) != 0
    }

    pub(super) fn tick_drag_autoscroll(&mut self) -> bool {
        let Some((col, row)) = self.mouse_drag_pos else {
            return false;
        };
        self.handle_mouse_drag(row, col)
    }

    pub(super) fn scroll_viewport(&mut self, delta_lines: i32) {
        if delta_lines == 0 {
            return;
        }
        let i = self.active;
        let doc = &mut self.documents[i];
        let visible_rows = doc.view.layout.text.height as usize;
        if visible_rows == 0 {
            return;
        }
        let visible_map = crate::view::visible_line_indices(&doc.buffer, &doc.folds);
        doc.view
            .scroll_by(delta_lines, visible_rows, visible_map.len());
    }

    /// Shift+方向键或 mouse 扩展选区；extend_selection 为 true 时保留 anchor
    pub(super) fn move_cursor<F>(&mut self, extend_selection: bool, f: F)
    where
        F: FnOnce(&mut crate::cursor::Cursor, &crate::buffer::GapBuffer),
    {
        let doc = self.active_doc_mut();
        doc.view.follow_cursor = true;
        if extend_selection {
            if !doc.selection.is_active() {
                doc.selection.begin(doc.cursor);
            }
        } else {
            doc.selection.clear();
        }
        f(&mut doc.cursor, &doc.buffer);
    }
}
