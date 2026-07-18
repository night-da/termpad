//! 应用层 stub：Commit 07 接入 handle

mod terminal;

use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::command::Command;
use crate::document::Document;
use crate::error::{EditorError, EditorResult};
use crate::input::map_key;
use crate::search::SearchState;
use crate::view::{draw, EditorChrome, RenderContext};

pub use terminal::run_cli;

pub struct App {
    pub documents: Vec<Document>,
    pub active: usize,
    pub mode: crate::command::EditorMode,
    pub search: SearchState,
    pub status: String,
    pub prompt: String,
    pub word_hits: Vec<crate::search::Match>,
    pub should_quit: bool,
}

impl App {
    pub fn new(path: Option<PathBuf>) -> EditorResult<Self> {
        let mut documents = Vec::new();
        if let Some(p) = path {
            match crate::document::load_document(&p) {
                Ok(doc) => documents.push(doc),
                Err(EditorError::NotFound) => documents.push(Document::new_empty(Some(p))),
                Err(e) => return Err(e),
            }
        } else {
            documents.push(Document::new_empty(None));
        }
        Ok(Self {
            documents,
            active: 0,
            mode: crate::command::EditorMode::Normal,
            search: SearchState::default(),
            status: "Ctrl+S save | Ctrl+Q quit".into(),
            prompt: String::new(),
            word_hits: Vec::new(),
            should_quit: false,
        })
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> EditorResult<()> {
        let mut needs_redraw = true;
        while !self.should_quit {
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

            if event::poll(Duration::from_millis(250))
                .map_err(|e| EditorError::Io(e.to_string()))?
            {
                if let Event::Key(key) =
                    event::read().map_err(|e| EditorError::Io(e.to_string()))?
                {
                    if key.kind == KeyEventKind::Press {
                        if map_key(self.mode, key) == Command::Quit {
                            self.should_quit = true;
                        }
                        needs_redraw = true;
                    }
                }
            }
        }
        Ok(())
    }
}
