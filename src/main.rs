//! termpad — 轻量终端文本编辑器（Gap Buffer + ratatui TUI）
//!
//! 本文件仅为 CLI 入口；事件循环见 crate::app

mod app;
mod buffer;
mod command;
mod cursor;
mod document;
mod encoding;
mod error;
mod fold;
mod input;
mod search;
mod selection;
mod syntax;
mod theme;
mod view;

use std::env;
use std::path::PathBuf;

use app::run_cli;

#[cfg(test)]
mod smoke {
    use crate::buffer::GapBuffer;

    #[test]
    fn gap_roundtrip() {
        let buf = GapBuffer::from_str("hello");
        assert_eq!(buf.as_text(), "hello");
    }
}

fn main() {
    let path = env::args().nth(1).map(PathBuf::from);
    if let Err(e) = run_cli(path) {
        eprintln!("termpad error: {e}");
        std::process::exit(1);
    }
}
