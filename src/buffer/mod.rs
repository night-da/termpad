//! 文本存储层：字节导向的 Gap Buffer
//!
//! 上层用字符列表示光标/选区，本模块部分 API 返回/接受字节偏移，调用时注意各函数文档
mod gap;

pub use gap::GapBuffer;
