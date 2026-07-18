//! 单标签页文档（Commit 03 分阶段版）
//!
//! Commit 08 增加 delete_selection；Commit 09 增加 goto_match；
//! Commit 10 将 Language 迁至 syntax；Commit 14 增加 convert_line_endings / toggle_fold

use std::path::{Path, PathBuf};

use crate::buffer::GapBuffer;
use crate::cursor::Cursor;
use crate::encoding::{Encoding, LineEnding};
use crate::error::{EditorError, EditorResult};
use crate::fold::FoldState;
use crate::selection::Selection;
use crate::view::ViewState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Plain,
}

#[derive(Debug)]
pub struct Document {
    pub buffer: GapBuffer,
    pub cursor: Cursor,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    pub view: ViewState,
    pub lang: Language,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    pub folds: FoldState,
    pub selection: Selection,
}

impl Document {
    pub fn new_empty(path: Option<PathBuf>) -> Self {
        Self {
            buffer: GapBuffer::new(),
            cursor: Cursor::new(),
            path,
            dirty: false,
            view: ViewState::new(),
            lang: Language::Plain,
            encoding: Encoding::Utf8,
            line_ending: LineEnding::Lf,
            folds: FoldState,
            selection: Selection::default(),
        }
    }

    pub fn display_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[No Name]")
            .to_string()
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn tab_label(&self) -> String {
        let dirty = if self.dirty { "*" } else { "" };
        format!("{}{dirty}", self.display_name())
    }

    /// 跳转到搜索/单词匹配（m.col / m.len 为行内字节）
    pub fn goto_match(&mut self, m: &crate::search::Match) {
        self.cursor.set_from_offset(
            &self.buffer,
            self.buffer.line_byte_col_to_offset(m.row, m.col),
        );
    }

    /// 删除当前选区文本；无选区或空选区返回 false
    pub fn delete_selection(&mut self) -> bool {
        let head = self.cursor;
        let Some((start, end)) = self.selection.ordered_range(head) else {
            return false;
        };
        if start.row == end.row && start.col == end.col {
            self.selection.clear();
            return false;
        }
        let off_start = self.buffer.position_to_offset(start.row, start.col);
        let off_end = self.buffer.position_to_offset(end.row, end.col);
        self.buffer.delete_byte_range(off_start, off_end);
        self.cursor = start;
        self.cursor.clamp(&self.buffer);
        self.selection.clear();
        self.mark_dirty();
        true
    }
}

pub fn load_document(path: &Path) -> EditorResult<Document> {
    if !path.exists() {
        return Err(EditorError::NotFound);
    }
    let bytes = std::fs::read(path).map_err(EditorError::from)?;
    let (encoding, text) = Encoding::decode(&bytes).map_err(|e| EditorError::Io(e.to_string()))?;
    let line_ending = LineEnding::detect(&text);
    Ok(Document {
        buffer: GapBuffer::from_str(&text),
        cursor: Cursor::new(),
        path: Some(path.to_path_buf()),
        dirty: false,
        view: ViewState::new(),
        lang: Language::Plain,
        encoding,
        line_ending,
        folds: FoldState,
        selection: Selection::default(),
    })
}

pub fn save_document(doc: &Document) -> EditorResult<()> {
    let path = doc
        .path
        .as_ref()
        .ok_or_else(|| EditorError::Io("no file path to save".into()))?;
    let text = doc.line_ending.apply_to_text(&doc.buffer.as_text());
    let bytes = doc.encoding.encode_for_write(&text);
    std::fs::write(path, bytes).map_err(EditorError::from)
}
