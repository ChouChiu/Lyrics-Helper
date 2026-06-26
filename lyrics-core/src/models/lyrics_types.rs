use serde::{Deserialize, Serialize};

/// 歌词格式类型，用于标识已解析歌词的来源格式。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum LyricsTypes {
    /// 未知格式
    #[default]
    Unknown = 0,
    /// Lyricify 音节格式
    LyricifySyllable = 1,
    /// Lyricify 行格式
    LyricifyLines = 2,
    /// 标准 LRC 格式
    Lrc = 3,
    /// QQ 音乐 QRC 格式
    Qrc = 4,
    /// 酷狗音乐 KRC 格式
    Krc = 5,
    /// 网易云音乐 YRC 格式
    Yrc = 6,
    /// Apple Music TTML 格式
    Ttml = 7,
    /// Spotify 格式
    Spotify = 8,
    /// Musixmatch 格式
    Musixmatch = 9,
}

/// 歌词原始格式类型，包含 [`LyricsTypes`] 的所有变体以及额外的原始子格式。
///
/// 用于区分同一平台的不同数据变体（如 QRC 与 QRC 完整版）。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum LyricsRawTypes {
    /// 未知格式
    #[default]
    Unknown = 0,
    /// Lyricify 音节格式
    LyricifySyllable = 1,
    /// Lyricify 行格式
    LyricifyLines = 2,
    /// 标准 LRC 格式
    Lrc = 3,
    /// QQ 音乐 QRC 格式
    Qrc = 4,
    /// QQ 音乐 QRC 完整版格式（包含额外元数据）
    QrcFull = 10,
    /// 酷狗音乐 KRC 格式
    Krc = 5,
    /// 网易云音乐 YRC 格式
    Yrc = 6,
    /// 网易云音乐 YRC 完整版格式（包含额外元数据）
    YrcFull = 11,
    /// Apple Music TTML 格式
    Ttml = 7,
    /// Apple Music JSON 格式
    AppleJson = 12,
    /// Spotify 格式
    Spotify = 8,
    /// Musixmatch 格式
    Musixmatch = 9,
}

/// 歌词同步类型，描述歌词数据的时间同步精度。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SyncTypes {
    /// 未知同步类型
    #[default]
    Unknown = 0,
    /// 音节级同步（逐字/逐音节高亮）
    SyllableSynced = 1,
    /// 行级同步（逐行高亮）
    LineSynced = 2,
    /// 混合同步（部分行有音节数据，部分行仅有行时间戳）
    MixedSynced = 3,
    /// 无同步（纯文本歌词，无时间信息）
    Unsynced = 4,
}

/// 歌词文本对齐方式。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LyricsAlignment {
    /// 未指定对齐方式
    #[default]
    Unspecified,
    /// 左对齐
    Left,
    /// 右对齐
    Right,
}


