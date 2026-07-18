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
                self.mode = EditorMode::Insert;
                self.status = "INSERT".into();
            }
            Command::EnterNormal => {
                self.mode = EditorMode::Normal;
                self.prompt.clear();
                self.status = "NORMAL".into();
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
            Command::EnterColumnInsert => {
                self.mode = EditorMode::ColumnInsert;
                self.status = "Column insert (stub until commit 14)".into();
            }
            Command::InsertChar(ch) => {
                if self.mode == EditorMode::ColumnInsert {
                    insert_at_column(self.active_doc_mut(), ch);
                } else {
                    let doc = self.active_doc_mut();
                    let off = doc.cursor.offset(&doc.buffer);
                    doc.buffer.insert_char(off, ch);
                    doc.cursor.move_right(&doc.buffer);
                }
                self.active_doc_mut().mark_dirty();
            }
            Command::Backspace => {
                let doc = self.active_doc_mut();
                let off = doc.cursor.offset(&doc.buffer);
                doc.buffer.delete_char_before(off);
                doc.cursor.move_left(&doc.buffer);
                doc.mark_dirty();
            }
            Command::Delete => {
                let doc = self.active_doc_mut();
                let off = doc.cursor.offset(&doc.buffer);
                doc.buffer.delete_char(off);
                doc.cursor.clamp(&doc.buffer);
                doc.mark_dirty();
            }
            Command::Newline => {
                let doc = self.active_doc_mut();
                let off = doc.cursor.offset(&doc.buffer);
                doc.buffer.insert_newline(off);
                doc.cursor.row += 1;
                doc.cursor.col = 0;
                doc.mark_dirty();
            }
            _ => {}
        }
        let doc = &mut self.documents[self.active];
        doc.cursor.clamp(&doc.buffer);
    }
}
