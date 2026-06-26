use crate::models::{LyricsData, LyricsTypes};

/// 歌词生成器 trait，将 [`LyricsData`] 序列化为特定格式的歌词文本。
pub trait LyricsGenerator {
    /// 将歌词数据生成为目标格式的字符串，生成失败返回 `None`。
    fn generate(&self, data: &LyricsData) -> Option<String>;

    /// 返回该生成器对应的歌词类型。
    fn lyrics_type(&self) -> LyricsTypes;
}
