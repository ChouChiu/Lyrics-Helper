//! 歌词数据模型模块。
//!
//! 定义了歌词解析和生成所需的核心数据结构，包括歌词行、音节、
//! 歌曲元数据、歌词类型枚举以及顶层歌词数据容器。

pub mod lyrics_types;
pub mod line_info;
pub mod syllable_info;
pub mod track_metadata;
pub mod lyrics_data;

pub use lyrics_types::*;
pub use line_info::*;
pub use syllable_info::*;
pub use track_metadata::*;
pub use lyrics_data::*;
