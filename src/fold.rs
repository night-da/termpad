//! 折叠 stub：Commit 14 前不做任何折叠

use crate::buffer::GapBuffer;

#[derive(Debug, Clone, Default)]
pub struct FoldState;

impl FoldState {
    pub fn toggle(&mut self, _buffer: &GapBuffer, _row: usize) {}

    pub fn is_hidden(&self, _row: usize) -> bool {
        false
    }

    pub fn is_folded_start(&self, _row: usize) -> bool {
        false
    }

    pub fn refresh(&mut self, _buffer: &GapBuffer) {}
}
