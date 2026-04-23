use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum LangError {
    #[error("unexpected character at byte offset {offset}")]
    UnexpectedChar { offset: usize },

    #[error("unterminated string literal starting at byte offset {offset}")]
    UnterminatedString { offset: usize },

    #[error("invalid integer literal '{text}' at {span:?}: {source}")]
    BadInt {
        text: String,
        span: std::ops::Range<usize>,
        #[source]
        source: std::num::ParseIntError,
    },

    #[error("invalid float literal '{text}' at {span:?}: {source}")]
    BadFloat {
        text: String,
        span: std::ops::Range<usize>,
        #[source]
        source: std::num::ParseFloatError,
    },

    #[error("parse error: {msg} (at byte offset {offset})")]
    ParseError { msg: String, offset: usize },

    #[error("unexpected end of file")]
    UnexpectedEof,
}
