use crate::models::{LyricsData, LyricsTypes};

pub trait LyricsGenerator {
    fn generate(&self, data: &LyricsData) -> Option<String>;
    fn lyrics_type(&self) -> LyricsTypes;
}
