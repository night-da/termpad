//! 编辑器全局错误类型
//!
//! 库内禁止 panic/unwrap；IO 与业务错误统一走 EditorResult

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorError {
    /// 底层 IO 失败（读写、终端等）
    Io(String),

    /// 打开路径时文件不存在（load_document；CLI 建空文件走 App::new 另一分支）
    NotFound,
}

impl fmt::Display for EditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "io error: {msg}"),
            Self::NotFound => write!(f, "file not found"),
        }
    }
}

impl std::error::Error for EditorError {}

impl From<std::io::Error> for EditorError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

pub type EditorResult<T> = Result<T, EditorError>;
