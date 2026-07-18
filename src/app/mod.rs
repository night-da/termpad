//! 应用层 stub：Commit 06 接入 view::draw；Commit 07 接入 handle

mod terminal;

use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Terminal;

use crate::command::Command;
use crate::document::Document;
use crate::error::{EditorError, EditorResult};
use crate::input::map_key;

pub use terminal::run_cli;

pub struct App {
    pub documents: Vec<Document>,
    pub active: usize,
    pub mode: crate::command::EditorMode,
    pub status: String,
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
            status: "Press Ctrl+Q to quit".into(),
            should_quit: false,
        })
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> EditorResult<()> {
        while !self.should_quit {
            terminal
                .draw(|f| {
                    let msg = format!(
                        "termpad — {} — Ctrl+Q quit",
                        self.documents[self.active].display_name()
                    );
                    Paragraph::new(msg).render(f.area(), f.buffer_mut());
                })
                .map_err(|e| EditorError::Io(e.to_string()))?;

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
                    }
                }
            }
        }
        Ok(())
    }
}
