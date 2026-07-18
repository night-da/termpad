use crossterm::event::MouseEvent;

use super::App;

impl App {
    pub(super) fn handle_mouse(&mut self, _mouse: MouseEvent) -> bool {
        false
    }

    pub(super) fn move_cursor<F>(&mut self, extend_selection: bool, f: F)
    where
        F: FnOnce(&mut crate::cursor::Cursor, &crate::buffer::GapBuffer),
    {
        let doc = self.active_doc_mut();
        doc.view.follow_cursor = true;
        if !extend_selection {
            doc.selection.clear();
        }
        f(&mut doc.cursor, &doc.buffer);
    }
}
