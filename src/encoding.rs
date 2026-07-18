//! UTF-8 / BOM 与换行符检测、保存转换（MVP 仅 UTF-8 系列，非 UTF-8 读入会失败）
//!
//! 读：Encoding::decode；写：encode_for_write + LineEnding::apply_to_text

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
    Utf8Bom,
}

impl Encoding {
    pub fn label(self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Utf8Bom => "UTF-8 BOM",
        }
    }

    /// 去掉 BOM 后按 UTF-8 解码；带 EF BB BF 前缀时返回 Utf8Bom
    pub fn decode(bytes: &[u8]) -> Result<(Self, String), std::string::FromUtf8Error> {
        if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            String::from_utf8(bytes[3..].to_vec()).map(|s| (Self::Utf8Bom, s))
        } else {
            String::from_utf8(bytes.to_vec()).map(|s| (Self::Utf8, s))
        }
    }

    pub fn encode_for_write(self, text: &str) -> Vec<u8> {
        match self {
            Self::Utf8 => text.as_bytes().to_vec(),
            Self::Utf8Bom => {
                let mut out = vec![0xEF, 0xBB, 0xBF];
                out.extend_from_slice(text.as_bytes());
                out
            }
        }
    }
}

/// 保存时使用的换行风格；detect 见任意 \r\n 即判 CRLF（混合文件按 CRLF 处理）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
}

impl LineEnding {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lf => "LF",
            Self::CrLf => "CRLF",
        }
    }

    /// 全文是否存在 \r\n；有则判 CRLF，否则 LF
    pub fn detect(text: &str) -> Self {
        if text.contains("\r\n") {
            Self::CrLf
        } else {
            Self::Lf
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Lf => Self::CrLf,
            Self::CrLf => Self::Lf,
        }
    }

    pub fn apply_to_text(self, text: &str) -> String {
        let unified = text.replace("\r\n", "\n");
        match self {
            Self::Lf => unified,
            Self::CrLf => unified.replace('\n', "\r\n"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_bom() {
        let bytes = [0xEF, 0xBB, 0xBF, b'h', b'i'];
        let (enc, s) = Encoding::decode(&bytes).unwrap();
        assert_eq!(enc, Encoding::Utf8Bom);
        assert_eq!(s, "hi");
    }

    #[test]
    fn line_ending_toggle() {
        assert_eq!(LineEnding::Lf.toggle(), LineEnding::CrLf);
        assert_eq!(LineEnding::CrLf.apply_to_text("a\nb"), "a\r\nb".to_string());
    }
}
