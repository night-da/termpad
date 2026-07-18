//! ccpp_theme 暗色配色（移植自 ccpp_theme，https://github.com/xenkuo/ccpp_theme）
//!
//! 下方 CYAN 等语义名与 ccpp color-table.md 对应；KEYWORD/TYPE 等 alias 供语法高亮调用
//! 查找/选区底色见 FIND_*、SELECTION_BG（对应 ccpp_theme.json 的 editor.*）

use ratatui::style::Color;

pub struct CcppTheme;

impl CcppTheme {
    // 编辑器 chrome（ccpp_theme.json colors.*）
    pub const EDITOR_BG: Color = Color::Rgb(0x28, 0x2A, 0x36);
    pub const LINE_NUMBER: Color = Color::Rgb(0x62, 0x72, 0xA4);
    pub const LINE_NUMBER_ACTIVE: Color = Color::Rgb(0xF5, 0xF5, 0xEF);
    pub const GUTTER_MARK: Color = Color::Rgb(0x62, 0x72, 0xA4);

    pub const STATUS_BG: Color = Color::Rgb(0x19, 0x1A, 0x21);
    pub const STATUS_FG: Color = Color::Rgb(0xF5, 0xF5, 0xEF);
    pub const TAB_ACTIVE_BG: Color = Color::Rgb(0x42, 0x44, 0x50);
    pub const TAB_ACTIVE_FG: Color = Color::Rgb(0xF5, 0xF5, 0xEF);
    pub const TAB_INACTIVE_FG: Color = Color::Rgb(0x62, 0x72, 0xA4);

    // ccpp color-table.md 基础色（部分名称与视觉色不一致，以 RGB 为准）
    pub const WHITE: Color = Color::Rgb(0xF5, 0xF5, 0xEF);
    pub const CYAN: Color = Color::Rgb(0x5F, 0xEA, 0x77); // ccpp 关键字绿
    pub const DEEP_CYAN: Color = Color::Rgb(0x8B, 0xE9, 0xFD);
    pub const YELLOW: Color = Color::Rgb(0xEA, 0xF4, 0x8C);
    pub const YELLOW_BRIGHT: Color = Color::Rgb(0xE9, 0xF2, 0x84);
    pub const ORANGE: Color = Color::Rgb(0xFF, 0xB8, 0x6C);
    pub const MAGENTA: Color = Color::Rgb(0xFF, 0x79, 0xC6);
    pub const RED: Color = Color::Rgb(0xFF, 0x55, 0x55);
    pub const PURPLE: Color = Color::Rgb(0xBD, 0x93, 0xF9);
    pub const BROWN: Color = Color::Rgb(0xBC, 0xAA, 0xA4);
    pub const COMMENT: Color = Color::Rgb(0x62, 0x72, 0xA4);

    // 语法高亮 alias
    pub const PLAIN: Color = Self::WHITE;
    pub const KEYWORD: Color = Self::CYAN;
    pub const TYPE: Color = Self::DEEP_CYAN;
    pub const FUNCTION: Color = Self::CYAN;
    pub const MACRO: Color = Self::MAGENTA;
    pub const CONSTANT: Color = Self::PURPLE;
    pub const STRING: Color = Self::YELLOW;
    pub const STRING_DELIM: Color = Self::YELLOW_BRIGHT;
    pub const NUMBER: Color = Self::YELLOW;
    pub const PARAMETER: Color = Self::ORANGE;
    pub const STATIC_GLOBAL: Color = Self::RED;
    pub const PROPERTY: Color = Self::BROWN;
    pub const OPERATOR_LOGICAL: Color = Self::RED;
    pub const OPERATOR: Color = Self::CYAN;
    pub const PUNCTUATION: Color = Self::WHITE;
    pub const ESCAPE: Color = Self::MAGENTA;
    pub const LABEL: Color = Self::RED;
    pub const LANGUAGE_SPECIAL: Color = Self::PURPLE;

    // Markdown / 散文（ccpp_theme.json tokenColors）
    pub const HEADING: Color = Self::PURPLE;
    pub const BOLD: Color = Self::ORANGE;
    pub const ITALIC: Color = Self::YELLOW;
    pub const CODE: Color = Self::CYAN;
    pub const LINK: Color = Self::MAGENTA;
    pub const LINK_URL: Color = Self::DEEP_CYAN;
    pub const LIST_MARK: Color = Self::DEEP_CYAN;
    pub const QUOTE: Color = Self::YELLOW;
    pub const HORIZONTAL_RULE: Color = Self::COMMENT;

    pub const FIND_CURRENT_BG: Color = Color::Rgb(0xFF, 0xB8, 0x6C);
    pub const FIND_OTHER_BG: Color = Color::Rgb(0x5E, 0x5F, 0x68);
    pub const WORD_HIGHLIGHT_BG: Color = Color::Rgb(0x44, 0x47, 0x5A);
    pub const SELECTION_BG: Color = Color::Rgb(0x3A, 0x5A, 0x42);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_matches_ccpp_color_table() {
        assert_eq!(CcppTheme::KEYWORD, Color::Rgb(0x5F, 0xEA, 0x77));
        assert_eq!(CcppTheme::TYPE, Color::Rgb(0x8B, 0xE9, 0xFD));
        assert_eq!(CcppTheme::MACRO, Color::Rgb(0xFF, 0x79, 0xC6));
        assert_eq!(CcppTheme::PARAMETER, Color::Rgb(0xFF, 0xB8, 0x6C));
        assert_eq!(CcppTheme::STATIC_GLOBAL, Color::Rgb(0xFF, 0x55, 0x55));
    }
}
