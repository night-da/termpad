use crate::command::{Command, EditorMode};
use crate::document::save_document;

use super::edit::insert_at_column;
use super::App;

impl App {
    pub fn handle(&mut self, cmd: Command) {
        match cmd {
            Command::Quit => {
                self.should_quit = true;
            }
            Command::Save => match save_document(self.active_doc()) {
                Ok(()) => {
                    self.active_doc_mut().mark_clean();
                    self.status = "Saved".into();
                }
                Err(e) => self.status = format!("Save failed: {e}"),
            },
            Command::EnterInsert => {
                self.active_doc_mut().selection.clear();
                self.mode = EditorMode::Insert;
                self.status = "INSERT".into();
            }
            Command::EnterNormal => {
                self.mode = EditorMode::Normal;
                self.prompt.clear();
                self.status = "NORMAL".into();
            }
            Command::EnterSearchForward => {
                self.mode = EditorMode::SearchForward;
                self.search.forward = true;
                self.search.clear();
                self.prompt = "/".into();
                self.status = "Ctrl+R regex  Ctrl+I icase".into();
            }
            Command::EnterSearchBackward => {
                self.mode = EditorMode::SearchBackward;
                self.search.forward = false;
                self.search.clear();
                self.prompt = "?".into();
                self.status = "Ctrl+R regex  Ctrl+I icase".into();
            }
            Command::SearchToggleRegex => {
                self.search.toggle_regex();
                self.status = format!("Search: {}", self.search.options_label());
            }
            Command::SearchToggleCase => {
                self.search.toggle_case_insensitive();
                self.status = format!("Search: {}", self.search.options_label());
            }
            Command::MoveLeft => self.move_cursor(false, |c, b| c.move_left(b)),
            Command::MoveRight => self.move_cursor(false, |c, b| c.move_right(b)),
            Command::MoveUp => self.move_cursor(false, |c, b| c.move_up(b)),
            Command::MoveDown => self.move_cursor(false, |c, b| c.move_down(b)),
            Command::MoveHome => self.move_cursor(false, |c, _| c.move_home()),
            Command::MoveEnd => self.move_cursor(false, |c, b| c.move_end(b)),
            Command::PageUp => self.move_cursor(false, |c, b| {
                c.page_up(10);
                c.clamp(b);
            }),
            Command::PageDown => self.move_cursor(false, |c, b| {
                c.page_down(b, 10);
                c.clamp(b);
            }),
            Command::SelectLeft => self.move_cursor(true, |c, b| c.move_left(b)),
            Command::SelectRight => self.move_cursor(true, |c, b| c.move_right(b)),
            Command::SelectUp => self.move_cursor(true, |c, b| c.move_up(b)),
            Command::SelectDown => self.move_cursor(true, |c, b| c.move_down(b)),
            Command::SelectHome => self.move_cursor(true, |c, _| c.move_home()),
            Command::SelectEnd => self.move_cursor(true, |c, b| c.move_end(b)),
            Command::SelectPageUp => self.move_cursor(true, |c, b| {
                c.page_up(10);
                c.clamp(b);
            }),
            Command::SelectPageDown => self.move_cursor(true, |c, b| {
                c.page_down(b, 10);
                c.clamp(b);
            }),
            Command::MouseDown { row, col } => self.handle_mouse_down(row, col),
            Command::MouseDrag { row, col } => {
                let _ = self.handle_mouse_drag(row, col);
            }
            Command::MouseUp => {
                self.mouse_selecting = false;
                self.selection_drag_start = None;
                self.mouse_drag_pos = None;
            }
            Command::ScrollUp { lines } => self.scroll_viewport(-(lines as i32)),
            Command::ScrollDown { lines } => self.scroll_viewport(lines as i32),
            Command::EnterColumnInsert => {
                self.mode = EditorMode::ColumnInsert;
                self.status = "Column insert (stub until commit 14)".into();
            }
            Command::InsertChar(ch) => {
                let column_insert = self.mode == EditorMode::ColumnInsert;
                let doc = self.active_doc_mut();
                if doc.selection.is_active() {
                    doc.delete_selection();
                }
                if column_insert {
                    insert_at_column(doc, ch);
                } else {
                    let off = doc.cursor.offset(&doc.buffer);
                    doc.buffer.insert_char(off, ch);
                    doc.cursor.move_right(&doc.buffer);
                }
                doc.mark_dirty();
            }
            Command::Backspace => {
                let doc = self.active_doc_mut();
                if doc.selection.is_active() {
                    doc.delete_selection();
                    doc.mark_dirty();
                    return;
                }
                let off = doc.cursor.offset(&doc.buffer);
                doc.buffer.delete_char_before(off);
                doc.cursor.move_left(&doc.buffer);
                doc.mark_dirty();
            }
            Command::Delete => {
                let doc = self.active_doc_mut();
                if doc.selection.is_active() {
                    doc.delete_selection();
                    doc.mark_dirty();
                    return;
                }
                let off = doc.cursor.offset(&doc.buffer);
                doc.buffer.delete_char(off);
                doc.cursor.clamp(&doc.buffer);
                doc.mark_dirty();
            }
            Command::Newline => {
                let doc = self.active_doc_mut();
                if doc.selection.is_active() {
                    doc.delete_selection();
                }
                let off = doc.cursor.offset(&doc.buffer);
                doc.buffer.insert_newline(off);
                doc.cursor.row += 1;
                doc.cursor.col = 0;
                doc.mark_dirty();
            }
            Command::SearchInput(ch) => {
                self.search.query.push(ch);
                self.prompt.push(ch);
            }
            Command::SearchBackspace => {
                self.search.query.pop();
                self.prompt.pop();
            }
            Command::ExecuteSearch => {
                let i = self.active;
                self.search.compile(&self.documents[i].buffer);
                self.goto_match();
                self.mode = EditorMode::Normal;
                self.prompt.clear();
                self.status = format!("Found {} matches", self.search.matches.len());
            }
            Command::NextMatch => {
                self.search.forward = true;
                self.search.next_match();
                self.goto_match();
            }
            Command::PrevMatch => {
                self.search.forward = false;
                self.search.next_match();
                self.goto_match();
            }
            _ => {}
        }
        let doc = &mut self.documents[self.active];
        doc.cursor.clamp(&doc.buffer);
    }
}
