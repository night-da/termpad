//! 提示符：Goto line: / Open path: / Replace with: 前缀与 prompt 字符串绑定

use std::path::{Path, PathBuf};

use crate::command::EditorMode;
use crate::document::{load_document, Document};
use crate::error::EditorError;

use super::App;

impl App {
    /// 解析 Goto line: 前缀后的 1-based 行号并跳转
    pub(super) fn execute_goto(&mut self) {
        let raw = self.prompt.trim_start_matches("Goto line: ");
        if let Ok(line) = raw.trim().parse::<usize>() {
            if line >= 1 {
                let doc = self.active_doc_mut();
                doc.view.follow_cursor = true;
                doc.cursor.row = line - 1; // 用户输入 1-based
                doc.cursor.clamp(&doc.buffer);
                self.status = format!("Goto line {line}");
            }
        }
        self.mode = EditorMode::Normal;
        self.prompt.clear();
    }

    /// 打开已有文件或创建新标签；路径写入 Document.path
    pub(super) fn execute_open(&mut self) {
        let raw = self.prompt.trim_start_matches("Open path: ").trim();
        if raw.is_empty() {
            self.mode = EditorMode::Normal;
            self.prompt.clear();
            return;
        }
        let path = PathBuf::from(raw);
        match load_document(Path::new(raw)) {
            Ok(doc) => {
                self.documents.push(doc);
                self.active = self.documents.len() - 1;
                self.status = format!("Opened {raw}");
            }
            Err(EditorError::NotFound) => {
                self.documents.push(Document::new_empty(Some(path)));
                self.active = self.documents.len() - 1;
                self.status = format!("New file {raw}");
            }
            Err(e) => self.status = format!("Open failed: {e}"),
        }
        self.mode = EditorMode::Normal;
        self.prompt.clear();
    }
}
