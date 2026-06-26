pub use lyrics_core::*;
pub use lyrics_parsers as parsers;
pub use lyrics_generators as generators;
pub use lyrics_crypto as decrypter;

#[cfg(feature = "search")]
pub use lyrics_search as search;
#[cfg(feature = "search")]
pub use lyrics_search::searchers;
#[cfg(feature = "search")]
pub use lyrics_search::providers;

pub use lyrics_parsers::parsers::parse_lyrics as parse;
pub use lyrics_parsers::parsers::parse_lyrics_auto as parse_auto;
pub use lyrics_generators::generate_string;

pub use lyrics_crypto::decrypter::qrc::decrypter::decrypt_lyrics as decrypt_qrc;
pub use lyrics_crypto::decrypter::krc::decrypter::decrypt_lyrics as decrypt_krc;
