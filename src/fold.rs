//! {/} 块折叠，按字符扫括号，不跳过字符串/注释（MVP 简化，字符串内 { 可能误判）

use std::collections::HashSet;

use crate::buffer::GapBuffer;

#[derive(Debug, Clone, Default)]
pub struct FoldState {
    /// 已折叠块的起始逻辑行号
    pub folded_starts: HashSet<usize>,
    /// 由 folded_starts 推导的 (start, end) 区间，供 is_hidden 查询
    regions: Vec<(usize, usize)>,
}

impl FoldState {
    pub fn toggle(&mut self, buffer: &GapBuffer, row: usize) {
        if self.folded_starts.contains(&row) {
            self.folded_starts.remove(&row);
        } else if let Some((start, end)) = find_fold_region(buffer, row) {
            if end > start {
                self.folded_starts.insert(start);
            }
        }
        self.rebuild_regions(buffer);
    }

    /// 折叠头行 start 仍可见；隐藏 (start, end] 内行
    pub fn is_hidden(&self, row: usize) -> bool {
        self.regions
            .iter()
            .any(|&(start, end)| row > start && row <= end)
    }

    pub fn is_folded_start(&self, row: usize) -> bool {
        self.folded_starts.contains(&row)
    }

    fn rebuild_regions(&mut self, buffer: &GapBuffer) {
        self.regions.clear();
        for &start in &self.folded_starts {
            if let Some((s, e)) = find_fold_region(buffer, start) {
                self.regions.push((s, e));
            }
        }
    }

    pub fn refresh(&mut self, buffer: &GapBuffer) {
        self.rebuild_regions(buffer);
    }
}

/// 从 row 行起找首个 {，向下扫描直至括号深度归零
fn find_fold_region(buffer: &GapBuffer, row: usize) -> Option<(usize, usize)> {
    let line = buffer.line(row)?;
    if !line.contains('{') {
        return None;
    }
    let total = buffer.line_count();
    let mut depth = 0i32;
    for r in row..total {
        let text = buffer.line(r).unwrap_or_default();
        for ch in text.chars() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some((row, r));
                    }
                }
                _ => {}
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_hides_inner_lines() {
        let buf = GapBuffer::from_str("fn main() {\n    let x = 1;\n}\n");
        let mut fold = FoldState::default();
        fold.toggle(&buf, 0);
        assert!(fold.is_hidden(1));
        assert!(!fold.is_hidden(0));
    }
}
