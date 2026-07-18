//! 命令分发：Command → 文档 / 搜索 / 模式切换
//!
//! handle 末尾统一 clamp 光标；Normal 模式下刷新单词高亮

use crate::command::{Command, EditorMode};
use crate::document::save_document;

use super::edit::insert_at_column;
use super::App;

impl App {
    /// 单条 Command 的副作用入口；模式/文档/搜索/global 状态均在此更新
    pub fn handle(&mut self, cmd: Command) {
        match cmd {
            // --- 退出与标签关闭确认 ---
            Command::Quit => {
                if self.active_doc().dirty {
                    self.mode = EditorMode::QuitConfirm;
                    self.prompt = "Unsaved changes — y=quit  n=stay".into();
                } else {
                    self.should_quit = true;
                }
            }
            Command::QuitYes => self.should_quit = true,
            Command::QuitNo => {
                self.mode = EditorMode::Normal;
                self.prompt.clear();
            }
            Command::CloseTabYes => {
                self.close_active_tab();
                self.mode = EditorMode::Normal;
                self.prompt.clear();
            }
            Command::CloseTabNo => {
                self.mode = EditorMode::Normal;
                self.prompt.clear();
            }
            Command::Save => match save_document(self.active_doc()) {
                Ok(()) => {
                    self.active_doc_mut().mark_clean();
                    self.status = "Saved".into();
                }
                Err(e) => self.status = format!("Save failed: {e}"),
            },
            // --- 模式切换 ---
            Command::EnterInsert => {
                if true {
                    self.active_doc_mut().selection.clear();
                    self.mode = EditorMode::Insert;
                    self.status = "INSERT".into();
                }
            }
            Command::EnterColumnInsert => {
                if true {
                    self.mode = EditorMode::ColumnInsert;
                    self.status = "Column insert (Alt+C / Esc)".into();
                }
            }
            Command::EnterNormal => {
                self.mode = EditorMode::Normal;
                self.prompt.clear();
                self.status = "NORMAL".into();
                self.refresh_word_highlight();
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
            Command::EnterReplaceInput => {
                self.mode = EditorMode::ReplaceInput;
                self.prompt = "Replace with: ".into();
                self.replace_with.clear();
            }
            Command::EnterGotoLine => {
                self.mode = EditorMode::GotoLine;
                self.prompt = "Goto line: ".into();
            }
            Command::EnterOpenPath => {
                self.mode = EditorMode::OpenPath;
                self.prompt = "Open path: ".into();
            }
            Command::NewTab => {
                self.documents
                    .push(crate::document::Document::new_empty(None));
                self.active = self.documents.len() - 1;
                self.status = "New tab".into();
            }
            Command::NextTab => {
                if !self.documents.is_empty() {
                    self.active = (self.active + 1) % self.documents.len();
                }
            }
            Command::PrevTab => {
                if !self.documents.is_empty() {
                    self.active = (self.active + self.documents.len() - 1) % self.documents.len();
                }
            }
            Command::CloseTab => {
                if self.documents.len() <= 1 {
                    return;
                }
                if self.active_doc().dirty {
                    self.mode = EditorMode::CloseTabConfirm;
                    self.prompt = "Unsaved — y=close tab  n=stay".into();
                } else {
                    self.close_active_tab();
                }
            }
            Command::ToggleWhitespace => {
                self.active_doc_mut().view.toggle_whitespace();
            }
            Command::ToggleLineEnding => {
                let next = self.active_doc().line_ending.toggle();
                self.active_doc_mut().convert_line_endings(next);
                self.status = format!("Line ending: {}", next.label());
            }
            Command::ToggleFold => {
                self.active_doc_mut().toggle_fold_at_cursor();
            }
            Command::SearchToggleRegex => {
                self.search.toggle_regex();
                self.status = format!("Search: {}", self.search.options_label());
            }
            Command::SearchToggleCase => {
                self.search.toggle_case_insensitive();
                self.status = format!("Search: {}", self.search.options_label());
            }
            // --- 光标移动（ColumnInsert 上下时保持列） ---
            Command::MoveLeft => self.move_cursor(false, |c, b| c.move_left(b)),
            Command::MoveRight => self.move_cursor(false, |c, b| c.move_right(b)),
            Command::MoveUp => {
                let preserve_col = self.mode == EditorMode::ColumnInsert;
                self.move_cursor(false, |c, b| {
                    let col = if preserve_col { Some(c.col) } else { None };
                    c.move_up(b);
                    if let Some(saved) = col {
                        c.col = saved;
                    }
                });
            }
            Command::MoveDown => {
                let preserve_col = self.mode == EditorMode::ColumnInsert;
                self.move_cursor(false, |c, b| {
                    let col = if preserve_col { Some(c.col) } else { None };
                    c.move_down(b);
                    if let Some(saved) = col {
                        c.col = saved;
                    }
                });
            }
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
            // --- 鼠标与滚轮 ---
            Command::MouseDown { row, col } => self.handle_mouse_down(row, col),
            Command::MouseDrag { row, col } => {
                let _ = self.handle_mouse_drag(row, col);
            }
            Command::MouseUp => {
                self.mouse_selecting = false;
                self.selection_drag_start = None;
            }
            Command::ScrollUp { lines } => self.scroll_viewport(-(lines as i32)),
            Command::ScrollDown { lines } => self.scroll_viewport(lines as i32),
            // --- 文本编辑：有选区时先删再写 ---
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
            // --- 搜索输入与执行 ---
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
            Command::ReplaceCurrent => {
                let i = self.active;
                let replacement = self.replace_with.clone();
                if self
                    .search
                    .replace_current(&mut self.documents[i].buffer, &replacement)
                {
                    self.documents[i].mark_dirty();
                    self.status = "Replaced one".into();
                }
            }
            Command::ReplaceAll => {
                let i = self.active;
                let replacement = self.replace_with.clone();
                let n = self
                    .search
                    .replace_all(&mut self.documents[i].buffer, &replacement);
                self.documents[i].mark_dirty();
                self.status = format!("Replaced {n} matches");
            }
            // --- 提示符（跳转行 / 打开路径 / 替换串） ---
            Command::PromptInput(ch) => self.prompt.push(ch),
            Command::PromptBackspace => {
                self.prompt.pop();
            }
            Command::ExecutePrompt => match self.mode {
                EditorMode::GotoLine => self.execute_goto(),
                EditorMode::OpenPath => self.execute_open(),
                EditorMode::ReplaceInput => {
                    self.replace_with =
                        self.prompt.trim_start_matches("Replace with: ").to_string();
                    self.mode = EditorMode::ReplaceConfirm;
                    self.prompt = "y=one a=all n=next".into();
                    self.status = "Replace confirm".into();
                }
                _ => {}
            },
            Command::Noop => {}
        }
        let mode = self.mode;
        {
            let doc = &mut self.documents[self.active];
            doc.cursor.clamp(&doc.buffer);
        }
        if matches!(mode, EditorMode::Normal) {
            self.refresh_word_highlight();
        }
    }
}
