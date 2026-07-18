//! 键位映射：crossterm KeyEvent → Command
//!
//! 优先级：Ctrl 组合键 → Alt → 当前 EditorMode 下的普通键
//! Shift+方向键在 Normal/Insert 下映射为 Select*，用于扩展选区

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::command::{Command, EditorMode};

/// 根据当前模式将按键转为语义命令；未识别键返回 Noop
pub fn map_key(mode: EditorMode, key: KeyEvent) -> Command {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('s') | KeyCode::Char('S') => Command::Save,
            KeyCode::Char('q') | KeyCode::Char('Q') => Command::Quit,
            KeyCode::Char('f') | KeyCode::Char('F') => Command::EnterSearchForward,
            KeyCode::Char('g') | KeyCode::Char('G') => Command::EnterGotoLine,
            KeyCode::Char('o') | KeyCode::Char('O') => Command::EnterOpenPath,
            KeyCode::Char('t') | KeyCode::Char('T') => Command::NewTab,
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    Command::PrevTab
                } else {
                    Command::NextTab
                }
            }
            KeyCode::Char('w') | KeyCode::Char('W') => Command::ToggleWhitespace,
            KeyCode::Char('e') | KeyCode::Char('E') => Command::ToggleLineEnding,
            KeyCode::Char('r') | KeyCode::Char('R')
                if matches!(mode, EditorMode::SearchForward | EditorMode::SearchBackward) =>
            {
                Command::SearchToggleRegex
            }
            KeyCode::Char('i') | KeyCode::Char('I')
                if matches!(mode, EditorMode::SearchForward | EditorMode::SearchBackward) =>
            {
                Command::SearchToggleCase
            }
            _ => Command::Noop,
        };
    }

    if key.modifiers.contains(KeyModifiers::ALT) {
        return match key.code {
            KeyCode::Char('c') | KeyCode::Char('C') => Command::EnterColumnInsert,
            _ => Command::Noop,
        };
    }

    match mode {
        EditorMode::Normal => map_normal(key),
        EditorMode::Insert | EditorMode::ColumnInsert => map_insert(key),
        EditorMode::SearchForward | EditorMode::SearchBackward => map_search(key),
        EditorMode::ReplaceInput => map_prompt(key),
        EditorMode::ReplaceConfirm => map_replace(key),
        EditorMode::GotoLine | EditorMode::OpenPath => map_prompt(key),
        EditorMode::QuitConfirm => map_quit_confirm(key),
        EditorMode::CloseTabConfirm => map_close_tab_confirm(key),
    }
}

fn map_normal(key: KeyEvent) -> Command {
    // vi 风格：i 进入插入，/ ? 搜索，Shift+方向键扩展选区
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    match key.code {
        KeyCode::Char('i') => Command::EnterInsert,
        KeyCode::Char('/') => Command::EnterSearchForward,
        KeyCode::Char('?') => Command::EnterSearchBackward,
        KeyCode::Char(':') => Command::EnterReplaceInput,
        KeyCode::Char('n') => Command::NextMatch,
        KeyCode::Char('N') => Command::PrevMatch,
        KeyCode::Char('z') => Command::ToggleFold,
        KeyCode::Char('w') if key.modifiers.is_empty() => Command::CloseTab,
        KeyCode::Left if shift => Command::SelectLeft,
        KeyCode::Left => Command::MoveLeft,
        KeyCode::Right if shift => Command::SelectRight,
        KeyCode::Right => Command::MoveRight,
        KeyCode::Up if shift => Command::SelectUp,
        KeyCode::Up => Command::MoveUp,
        KeyCode::Down if shift => Command::SelectDown,
        KeyCode::Down => Command::MoveDown,
        KeyCode::Home if shift => Command::SelectHome,
        KeyCode::Home => Command::MoveHome,
        KeyCode::End if shift => Command::SelectEnd,
        KeyCode::End => Command::MoveEnd,
        KeyCode::PageUp if shift => Command::SelectPageUp,
        KeyCode::PageUp => Command::PageUp,
        KeyCode::PageDown if shift => Command::SelectPageDown,
        KeyCode::PageDown => Command::PageDown,
        _ => Command::Noop,
    }
}

fn map_insert(key: KeyEvent) -> Command {
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    match key.code {
        KeyCode::Esc => Command::EnterNormal,
        KeyCode::Enter => Command::Newline,
        KeyCode::Backspace => Command::Backspace,
        KeyCode::Delete => Command::Delete,
        KeyCode::Left if shift => Command::SelectLeft,
        KeyCode::Left => Command::MoveLeft,
        KeyCode::Right if shift => Command::SelectRight,
        KeyCode::Right => Command::MoveRight,
        KeyCode::Up if shift => Command::SelectUp,
        KeyCode::Up => Command::MoveUp,
        KeyCode::Down if shift => Command::SelectDown,
        KeyCode::Down => Command::MoveDown,
        KeyCode::Home if shift => Command::SelectHome,
        KeyCode::Home => Command::MoveHome,
        KeyCode::End if shift => Command::SelectEnd,
        KeyCode::End => Command::MoveEnd,
        KeyCode::PageUp if shift => Command::SelectPageUp,
        KeyCode::PageUp => Command::PageUp,
        KeyCode::PageDown if shift => Command::SelectPageDown,
        KeyCode::PageDown => Command::PageDown,
        KeyCode::Char(ch) if !shift => Command::InsertChar(ch),
        _ => Command::Noop,
    }
}

fn map_search(key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Esc => Command::EnterNormal,
        KeyCode::Enter => Command::ExecuteSearch,
        KeyCode::Backspace => Command::SearchBackspace,
        KeyCode::Up => Command::MoveUp,
        KeyCode::Down => Command::MoveDown,
        KeyCode::Left => Command::MoveLeft,
        KeyCode::Right => Command::MoveRight,
        KeyCode::Char(ch) => Command::SearchInput(ch),
        _ => Command::Noop,
    }
}

fn map_prompt(key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Esc => Command::EnterNormal,
        KeyCode::Enter => Command::ExecutePrompt,
        KeyCode::Backspace => Command::PromptBackspace,
        KeyCode::Char(ch) => Command::PromptInput(ch),
        _ => Command::Noop,
    }
}

fn map_replace(key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Esc => Command::EnterNormal,
        KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => Command::ReplaceCurrent,
        KeyCode::Char('a') | KeyCode::Char('A') => Command::ReplaceAll,
        KeyCode::Char('n') | KeyCode::Char('N') => Command::NextMatch,
        _ => Command::Noop,
    }
}

fn map_quit_confirm(key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => Command::QuitYes,
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Command::QuitNo,
        _ => Command::Noop,
    }
}

fn map_close_tab_confirm(key: KeyEvent) -> Command {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => Command::CloseTabYes,
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Command::CloseTabNo,
        _ => Command::Noop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn normal_i_enters_insert() {
        assert_eq!(
            map_key(
                EditorMode::Normal,
                key(KeyCode::Char('i'), KeyModifiers::NONE)
            ),
            Command::EnterInsert
        );
    }

    #[test]
    fn ctrl_s_saves_from_any_mode() {
        assert_eq!(
            map_key(
                EditorMode::Insert,
                key(KeyCode::Char('s'), KeyModifiers::CONTROL)
            ),
            Command::Save
        );
    }

    #[test]
    fn ctrl_tab_switches_tab() {
        assert_eq!(
            map_key(EditorMode::Normal, key(KeyCode::Tab, KeyModifiers::CONTROL)),
            Command::NextTab
        );
        assert_eq!(
            map_key(
                EditorMode::SearchForward,
                key(KeyCode::Tab, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
            ),
            Command::PrevTab
        );
    }

    #[test]
    fn search_mode_maps_movement_and_input() {
        assert_eq!(
            map_key(
                EditorMode::SearchForward,
                key(KeyCode::Up, KeyModifiers::NONE)
            ),
            Command::MoveUp
        );
        assert_eq!(
            map_key(
                EditorMode::SearchForward,
                key(KeyCode::Esc, KeyModifiers::NONE)
            ),
            Command::EnterNormal
        );
        assert_eq!(
            map_key(
                EditorMode::SearchForward,
                key(KeyCode::Char('x'), KeyModifiers::NONE)
            ),
            Command::SearchInput('x')
        );
    }

    #[test]
    fn search_mode_ctrl_r_toggles_regex() {
        assert_eq!(
            map_key(
                EditorMode::SearchForward,
                key(KeyCode::Char('r'), KeyModifiers::CONTROL)
            ),
            Command::SearchToggleRegex
        );
    }

    #[test]
    fn insert_esc_returns_to_normal() {
        assert_eq!(
            map_key(EditorMode::Insert, key(KeyCode::Esc, KeyModifiers::NONE)),
            Command::EnterNormal
        );
    }

    #[test]
    fn quit_confirm_yes_no() {
        assert_eq!(
            map_key(
                EditorMode::QuitConfirm,
                key(KeyCode::Char('y'), KeyModifiers::NONE)
            ),
            Command::QuitYes
        );
        assert_eq!(
            map_key(
                EditorMode::QuitConfirm,
                key(KeyCode::Esc, KeyModifiers::NONE)
            ),
            Command::QuitNo
        );
    }
}
