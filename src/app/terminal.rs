//! 终端 raw 模式、备用屏幕、鼠标捕获；Drop 时恢复终端

use std::io::{self, stdout, Stdout};
use std::path::PathBuf;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::error::{EditorError, EditorResult};

use super::App;

pub struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    /// 进入 TUI 环境；失败时返回 EditorError::Io
    pub fn setup() -> EditorResult<Self> {
        enable_raw_mode().map_err(|e| EditorError::Io(e.to_string()))?;
        stdout()
            .execute(EnterAlternateScreen)
            .and_then(|s| s.execute(EnableMouseCapture))
            .map_err(|e| EditorError::Io(e.to_string()))?;
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend).map_err(|e| EditorError::Io(e.to_string()))?;
        Ok(Self { terminal })
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = stdout().execute(DisableMouseCapture);
        let _ = stdout().execute(LeaveAlternateScreen);
    }
}

pub fn run_app(path: Option<PathBuf>) -> EditorResult<()> {
    let mut guard = TerminalGuard::setup()?;
    let mut app = App::new(path)?;
    app.run(guard.terminal_mut())?;
    Ok(())
}

pub fn run_cli(path: Option<PathBuf>) -> io::Result<()> {
    match run_app(path) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("termpad error: {e}");
            Err(io::Error::other(e.to_string()))
        }
    }
}
