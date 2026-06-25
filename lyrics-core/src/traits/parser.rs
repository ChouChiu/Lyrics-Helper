use crate::models::{LyricsData, LyricsRawTypes};

pub trait LyricsParser {
    fn parse(&self, input: &str) -> Option<LyricsData>;
    fn raw_type(&self) -> LyricsRawTypes;
}
