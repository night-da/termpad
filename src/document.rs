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
            folds: FoldState::default(),
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
        folds: FoldState::default(),
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
