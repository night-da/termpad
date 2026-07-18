//! termpad — 轻量终端文本编辑器（Gap Buffer + ratatui TUI）
//!
//! 本文件仅为 CLI 入口；事件循环见 crate::app，单标签页状态见 crate::document
//! 可选第一个参数为待打开文件路径

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
mod word;

use std::env;
use std::path::PathBuf;

use app::run_cli;

fn main() {
    let path = env::args().nth(1).map(PathBuf::from);
    if let Err(e) = run_cli(path) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
