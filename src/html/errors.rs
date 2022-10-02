use crate::html::tokenizer::{Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("open & close tag name miss matched (open: {open:?}, close: {close:?})")]
    TagMissMatch { open: String, close: String },
    #[error("unexpected token: (expected: {expected:?}, found: {found:?})")]
    UnexpectedToken {
        expected: TokenKind,
        found: Token,
    },
    #[error("unexpected text: (expected: {expected:?}, found: {found:?})")]
    UnexpectedText {
        expected: String,
        found: Option<Box<Token>>,
    },
    #[error("unknown parse error")]
    Unknown,
}
