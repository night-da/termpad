//! 应用层：单线程事件循环、终端生命周期、输入分发与命令处理
//!
//! 无跨线程共享可变状态，子模块：handle（命令）、mouse、edit、prompt、terminal
//! search 跨标签共享；拖拽选区在边缘时缩短 poll 间隔以实现持续自动滚动

mod edit;
mod handle;
mod mouse;
mod prompt;
mod terminal;

use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::command::EditorMode;
use crate::document::Document;
use crate::error::{EditorError, EditorResult};
use crate::input::map_key;
use crate::search::SearchState;
use crate::view::{draw, EditorChrome, RenderContext};

pub use terminal::run_cli;

pub struct App {
    pub documents: Vec<Document>,
    pub active: usize,
    pub mode: EditorMode,
    /// 跨标签共享；切换标签时 on_active_tab_changed 会 clear
    pub search: SearchState,
    pub status: String,
    /// 底部提示或搜索/跳转输入缓冲
    pub prompt: String,
    pub replace_with: String,
    /// Normal 模式下光标处单词的全文匹配列表
    pub word_hits: Vec<crate::search::Match>,
    pub should_quit: bool,
    pub mouse_selecting: bool,
    pub selection_drag_start: Option<crate::cursor::Cursor>,

    /// 边缘停驻无新鼠标事件时，仍用上次坐标继续滚动选区
    pub mouse_drag_pos: Option<(u16, u16)>,
}

impl App {
    pub fn new(path: Option<PathBuf>) -> EditorResult<Self> {
        let mut documents = Vec::new();
        if let Some(p) = path {
            // 路径不存在时仍打开空白标签，便于「先编辑后保存」
            match crate::document::load_document(&p) {
                Ok(doc) => documents.push(doc),
                Err(EditorError::NotFound) => documents.push(Document::new_empty(Some(p))),
                Err(e) => return Err(e),
            }
        } else {
            documents.push(Document::new_empty(None));
        }
        let status = "Ctrl+S save | Ctrl+F find | Ctrl+G goto | Ctrl+Tab tabs".into();
        Ok(Self {
            documents,
            active: 0,
            mode: EditorMode::Normal,
            search: SearchState::default(),
            status,
            prompt: String::new(),
            replace_with: String::new(),
            word_hits: Vec::new(),
            should_quit: false,
            mouse_selecting: false,
            selection_drag_start: None,
            mouse_drag_pos: None,
        })
    }

    pub(crate) fn active_doc(&self) -> &Document {
        &self.documents[self.active]
    }

    pub(crate) fn active_doc_mut(&mut self) -> &mut Document {
        &mut self.documents[self.active]
    }

    pub(crate) fn refresh_word_highlight(&mut self) {
        // Commit 14 接入 word.rs 后填充 word_hits
        self.word_hits.clear();
    }

    /// 将 search.current 对应位置设为活动文档光标并开启 follow
    pub(crate) fn goto_match(&mut self) {
        if let Some(m) = self.search.current_match().cloned() {
            let doc = self.active_doc_mut();
            doc.view.follow_cursor = true;
            doc.goto_match(&m);
        }
    }

    /// 切换标签或新建后：清空搜索并退出搜索相关模式
    pub(crate) fn on_active_tab_changed(&mut self) {
        // 搜索状态全局共享，切换标签时必须清空，避免跨文件匹配残留
        self.search.clear();
        if matches!(
            self.mode,
            EditorMode::SearchForward
                | EditorMode::SearchBackward
                | EditorMode::ReplaceInput
                | EditorMode::ReplaceConfirm
        ) {
            self.mode = EditorMode::Normal;
            self.prompt.clear();
        }
        self.refresh_word_highlight();
    }

    pub(crate) fn close_active_tab(&mut self) {
        if self.documents.len() > 1 {
            self.documents.remove(self.active);
            self.active = self.active.min(self.documents.len() - 1);
            self.on_active_tab_changed();
        }
    }

    /// 主循环：按需 draw + poll 事件；拖拽边缘时缩短 poll 间隔
    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> EditorResult<()> {
        self.refresh_word_highlight();
        let mut needs_redraw = true;
        while !self.should_quit {
            // 按需重绘：避免固定周期 draw 重置终端光标闪烁相位
            if needs_redraw {
                let tab_labels: Vec<String> =
                    self.documents.iter().map(|d| d.tab_label()).collect();
                let chrome = EditorChrome {
                    tab_labels: &tab_labels,
                    active: self.active,
                    mode: self.mode,
                    search: &self.search,
                    status: &self.status,
                    prompt: &self.prompt,
                    word_hits: &self.word_hits,
                };
                terminal
                    .draw(|f| {
                        let doc = &mut self.documents[self.active];
                        draw(f, RenderContext { doc, chrome });
                    })
                    .map_err(|e| EditorError::Io(e.to_string()))?;
                needs_redraw = false;
            }
            // 拖拽选区在边缘时缩短 poll 间隔，实现持续自动滚动
            let poll_ms = if self.drag_autoscroll_active() {
                35
            } else {
                250
            };

            if event::poll(Duration::from_millis(poll_ms))
                .map_err(|e| EditorError::Io(e.to_string()))?
            {
                match event::read().map_err(|e| EditorError::Io(e.to_string()))? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        let cmd = map_key(self.mode, key);
                        self.handle(cmd);
                        needs_redraw = true;
                    }
                    Event::Mouse(mouse) => {
                        if self.handle_mouse(mouse) {
                            needs_redraw = true;
                        }
                    }
                    Event::Resize(_, _) => needs_redraw = true,
                    _ => {}
                }
            } else if self.mouse_selecting && self.tick_drag_autoscroll() {
                // 鼠标停于边缘无新事件时，仍按上次坐标继续滚动选区
                needs_redraw = true;
            }
        }
        Ok(())
    }
}
