//! 单标签页文档：buffer、光标、编码、折叠及内嵌视口状态
//!
//! 一个 Document 对应一个标签页，view 保存滚动/跟随光标等视口状态（与 crate::view 渲染逻辑分离，存在层耦合，见架构文档）
//! load_document 全量读入 Gap Buffer；路径不存在时由 App::new 建空白标签

use std::path::{Path, PathBuf};

use crate::buffer::GapBuffer;
use crate::cursor::Cursor;
use crate::encoding::{Encoding, LineEnding};
use crate::error::{EditorError, EditorResult};
use crate::fold::FoldState;
use crate::selection::Selection;
use crate::syntax::{detect_language, Language};
use crate::view::ViewState;

#[derive(Debug)]
pub struct Document {
    pub buffer: GapBuffer,
    pub cursor: Cursor,
    pub path: Option<PathBuf>,
    /// 自上次保存或加载后 buffer 有改动
    pub dirty: bool,
    /// 滚动、布局矩形、是否跟随光标等视口状态
    pub view: ViewState,
    pub lang: Language,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    pub folds: FoldState,
    pub selection: Selection,
}

impl Document {
    pub fn new_empty(path: Option<PathBuf>) -> Self {
        let lang = detect_language(path.as_deref());
        Self {
            buffer: GapBuffer::new(),
            cursor: Cursor::new(),
            path,
            dirty: false,
            view: ViewState::new(),
            lang,
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

    /// 标签栏显示名；未保存时前缀 *
    pub fn tab_label(&self) -> String {
        let dirty = if self.dirty { "*" } else { "" };
        format!("{}{dirty}", self.display_name())
    }

    pub fn toggle_fold_at_cursor(&mut self) {
        let row = self.cursor.row;
        self.folds.toggle(&self.buffer, row);
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

    pub fn convert_line_endings(&mut self, to: LineEnding) {
        // 整 buffer 重建，避免 gap 内逐字节替换 \r\n 的边界问题
        let text = self.buffer.as_text();
        let converted = to.apply_to_text(&text);
        self.buffer = GapBuffer::from_str(&converted);
        self.line_ending = to;
        self.mark_dirty();
        self.folds.refresh(&self.buffer);
        self.cursor.clamp(&self.buffer);
    }
}

/// 从磁盘读入全文；路径必须已存在（不存在时 App::new 走 new_empty）
pub fn load_document(path: &Path) -> EditorResult<Document> {
    if !path.exists() {
        return Err(EditorError::NotFound);
    }
    let bytes = std::fs::read(path).map_err(EditorError::from)?;
    let (encoding, text) = Encoding::decode(&bytes).map_err(|e| EditorError::Io(e.to_string()))?;
    let line_ending = LineEnding::detect(&text);
    let mut doc = Document {
        buffer: GapBuffer::from_str(&text),
        cursor: Cursor::new(),
        path: Some(path.to_path_buf()),
        dirty: false,
        view: ViewState::new(),
        lang: detect_language(Some(path)),
        encoding,
        line_ending,
        folds: FoldState::default(),
        selection: Selection::default(),
    };
    doc.folds.refresh(&doc.buffer);
    Ok(doc)
}

/// 按文档 encoding / line_ending 写回 path；无 path 时返回 Io 错误
pub fn save_document(doc: &Document) -> EditorResult<()> {
    let path = doc
        .path
        .as_ref()
        .ok_or_else(|| EditorError::Io("no file path to save".into()))?;
    let text = doc.line_ending.apply_to_text(&doc.buffer.as_text());
    let bytes = doc.encoding.encode_for_write(&text);
    std::fs::write(path, bytes).map_err(EditorError::from)
}
