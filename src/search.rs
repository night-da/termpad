//! 搜索 stub：Commit 09 整文件替换为终态 search.rs

use crate::buffer::GapBuffer;

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchOptions {
    pub use_regex: bool,
    pub case_insensitive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub row: usize,
    pub col: usize,
    pub len: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub query: String,
    pub forward: bool,
    pub options: SearchOptions,
    pub matches: Vec<Match>,
    pub current: Option<usize>,
}

impl SearchState {
    pub fn clear(&mut self) {
        self.query.clear();
        self.matches.clear();
        self.current = None;
    }

    pub fn options_label(&self) -> String {
        "literal".into()
    }

    pub fn compile(&mut self, _buffer: &GapBuffer) {}

    pub fn current_match(&self) -> Option<&Match> {
        None
    }
}
