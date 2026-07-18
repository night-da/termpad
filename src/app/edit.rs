//! 列插入模式（Alt+C）：在同一字符列向下连续写入，不足则 pad 空格
//!
//! 与 Normal 移动不同：输入后光标落到下一行同列，而非行尾 clamp

use crate::document::Document;

/// 在当前 (row, col) 写入 ch，光标落到 (row+1, col)
pub(crate) fn insert_at_column(doc: &mut Document, ch: char) {
    let row = doc.cursor.row;
    let col = doc.cursor.col;
    let line_len = doc.buffer.line_len(row);
    if line_len < col {
        let pad = col - line_len;
        let off = doc.buffer.position_to_offset(row, line_len);
        doc.buffer.insert_str(off, &" ".repeat(pad));
    }
    let off = doc.buffer.position_to_offset(row, col);
    doc.buffer.insert_char(off, ch);
    doc.cursor.row += 1;
    doc.cursor.col = col;
    doc.cursor.clamp(&doc.buffer);
}
