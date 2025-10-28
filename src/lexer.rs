use crate::source_code::SourceCode;
use crate::types::Token;
use core::iter::FusedIterator;

// N.B.: not all LexerErrors equal themselves as they could be originating from different places.
// therefore we don't implement `Eq` because we aren't reflexive (a != a).
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
#[non_exhaustive]
pub enum LexerError {
    UnexpectedEofWhile(Token),
    WithMessage(&'static str),
    InvalidEscapeSequence,
    InvalidCharacter,
    NoLiteralToExtract,
    Eof,

    Internal,
}

#[doc(hidden)]
#[macro_export]
macro_rules! lexer_error_here {
    ($message: literal) => {{
        $crate::lexer::LexerError::WithMessage(::core::concat!(
            "lexer error at ",
            ::core::file!(),
            ":",
            ::core::line!(),
            ":",
            ::core::column!(),
            ":\n\t",
        ))
    }};
}
pub use crate::lexer_error_here;

pub type LexerResult<T> = Result<T, LexerError>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lexer<'source> {
    source: SourceCode<'source>,
    start: usize,
    index: usize,

    literal: Option<&'source [u8]>,

    line: usize,
    column: usize,
}

mod lexer_impls;

impl<'source> Lexer<'source> {
    #[inline]
    pub const fn new(source: SourceCode<'source>) -> Self {
        Lexer {
            source,
            start: 0,
            index: 0,

            literal: None,

            line: 0,
            column: 0,
        }
    }

    /// After this function returns, you may be at the end.
    pub const fn lex_single_token(&mut self) -> LexerResult<Token> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Err(LexerError::Eof);
        }

        self.start = self.index;

        let next = unsafe { self.advance_unchecked() };
        let tok = match next {
            b'.' => Token::PuncDot,
            b',' => Token::PuncComma,
            b';' => Token::PuncSemi,
            b':' => Token::PuncColon,
            b'+' => Token::PuncPlus,
            b'*' => Token::PuncStar,
            b'/' => Token::PuncSlash,

            b'-' => {
                if let Some(cond) = self.matches(b'>')
                    && cond
                {
                    Token::PuncArrowRight
                } else {
                    Token::PuncMinus
                }
            }

            b'=' => {
                if let Some(cond) = self.matches(b'=')
                    && cond
                {
                    Token::PuncEqEq
                } else {
                    Token::PuncEq
                }
            }

            b'!' => {
                if let Some(cond) = self.matches(b'=')
                    && cond
                {
                    Token::PuncBangEq
                } else {
                    Token::PuncBang
                }
            }

            b'<' => match self.peek() {
                Some(b'=') => Token::PuncLtEq,
                Some(b'<') => Token::PuncShl,
                _ => Token::PuncLt,
            },

            b'>' => match self.peek() {
                Some(b'=') => Token::PuncGtEq,
                Some(b'>') => Token::PuncShr,
                _ => Token::PuncGt,
            },

            b'(' => Token::IndentLParen,
            b')' => Token::IndentRParen,
            b'{' => Token::IndentLBrace,
            b'}' => Token::IndentRBrace,
            b'[' => Token::IndentLBracket,
            b']' => Token::IndentRBracket,

            b'"' => {
                // SAFETY: self.index is always 1 character ahead of self.start due
                // to fixed advance unchecked
                match unsafe { self.parse_quoted_string() } {
                    Ok(tok) => tok,
                    Err(e) => return Err(e),
                }
            }

            b'\'' => {
                // SAFETY: self.index is always 1 character ahead of self.start due
                // to fixed advance unchecked
                match unsafe { self.parse_character_literal() } {
                    Ok(tok) => tok,
                    Err(e) => return Err(e),
                }
            }

            // todo +=, -= etc. operators

            // // todo: hex number literals
            // b'0' => {
            //     // handle 0x number literals
            //     if let Some(cond) = self.matches(b'x')
            //         && cond
            //     {
            //         unsafe {
            //             self.advance_unchecked();
            //             self.advance_unchecked();
            //         };

            //         // SAFETY: self.index is always 1 character ahead of self.start due
            //         // to fixed advance unchecked
            //         match unsafe { self.parse_ambiguous_number_literal(true) } {
            //             Ok(tok) => tok,
            //             Err(e) => return Err(e),
            //         }
            //     }
            // }
            c if lexer_impls::numbers::is_valid_digit(c) => {
                // SAFETY: self.index is always 1 character ahead of self.start due
                // to fixed advance unchecked
                match unsafe { self.parse_ambiguous_number_literal() } {
                    Ok(tok) => tok,
                    Err(e) => return Err(e),
                }
            }

            c if lexer_impls::identifiers::is_valid_identifier_head(c) => {
                // SAFETY: self.index is always 1 character ahead of self.start due
                // to fixed advance unchecked, and character validity is determined by
                // `is_valid_identifier_head`
                match unsafe { self.parse_identifier() } {
                    Ok(tok) => tok,
                    Err(e) => return Err(e),
                }
            }

            _ => return Err(LexerError::InvalidCharacter),
        };

        Ok(tok)
    }

    #[inline]
    pub const fn extract_literal(&mut self) -> LexerResult<&'source [u8]> {
        match self.literal.take() {
            Some(t) => Ok(t),
            None => Err(LexerError::NoLiteralToExtract),
        }
    }

    #[inline]
    pub const fn get_line_column(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    #[inline]
    pub const fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub const fn index(&self) -> usize {
        self.index
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Token;

    #[inline]
    fn next(&mut self) -> Option<Token> {
        self.lex_single_token().ok()
    }
}

impl FusedIterator for Lexer<'_> {}
