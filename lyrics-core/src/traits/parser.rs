use crate::models::{LyricsData, LyricsRawTypes};

/// 歌词解析器 trait，各格式解析器需实现此接口。
pub trait LyricsParser {
    /// 将原始歌词文本解析为 [`LyricsData`]，解析失败返回 `None`。
    fn parse(&self, input: &str) -> Option<LyricsData>;

    /// 返回该解析器对应的原始歌词格式类型。
    fn raw_type(&self) -> LyricsRawTypes;
}
