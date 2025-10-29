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
    UnclosedCharLiteral,
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

    // TODO: feature gate these bastards so backtracking and advance doesnt take a billion years
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

            line: 1,
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
        self.literal = None;

        let next = unsafe { self.advance_unchecked() };
        let tok = match next {
            b'.' => Token::PuncDot,
            b',' => Token::PuncComma,
            b';' => Token::PuncSemi,
            b':' => Token::PuncColon,

            b'+' => match self.peek() {
                Some(b'=') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncPlusEq
                }
                _ => Token::PuncPlus,
            },
            b'*' => match self.peek() {
                Some(b'=') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncStarEq
                }
                _ => Token::PuncStar,
            },
            b'/' => match self.peek() {
                Some(b'=') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncSlashEq
                }
                _ => Token::PuncSlash,
            },

            b'-' => match self.peek() {
                Some(b'>') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncArrowRight
                }
                Some(b'=') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncMinusEq
                }
                _ => Token::PuncMinus,
            },

            b'=' => match self.peek() {
                Some(b'=') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncEqEq
                }
                _ => Token::PuncEq,
            },

            b'!' => match self.peek() {
                Some(b'=') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncBangEq
                }
                _ => Token::PuncBang,
            },

            b'<' => match self.peek() {
                Some(b'=') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncLtEq
                }
                Some(b'<') => {
                    unsafe { self.advance_unchecked() };
                    match self.peek() {
                        Some(b'=') => {
                            unsafe { self.advance_unchecked() };
                            Token::PuncShlEq
                        }
                        _ => Token::PuncShl,
                    }
                }
                _ => Token::PuncLt,
            },

            b'>' => match self.peek() {
                Some(b'=') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncGtEq
                }
                Some(b'>') => {
                    unsafe { self.advance_unchecked() };
                    match self.peek() {
                        Some(b'=') => {
                            unsafe { self.advance_unchecked() };
                            Token::PuncShrEq
                        }
                        _ => Token::PuncShr,
                    }
                }
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
                match unsafe { self.lex_quoted_string() } {
                    Ok(tok) => tok,
                    Err(e) => return Err(e),
                }
            }

            b'\'' => {
                // SAFETY: self.index is always 1 character ahead of self.start due
                // to fixed advance unchecked
                match unsafe { self.lex_character_literal() } {
                    Ok(tok) => tok,
                    Err(e) => return Err(e),
                }
            }

            // todo +=, -= etc. operators

            // // todo: hex and octal number literals
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
            //         match unsafe { self.lex_ambiguous_number_literal(true) } {
            //             Ok(tok) => tok,
            //             Err(e) => return Err(e),
            //         }
            //     }
            // }
            //
            b'%' => match self.peek() {
                Some(b'=') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncModuloEq
                }
                _ => Token::PuncModulo,
            },

            b'&' => match self.peek() {
                Some(b'=') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncAndEq
                }
                _ => Token::PuncAnd,
            },

            b'|' => match self.peek() {
                Some(b'=') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncOrEq
                }
                _ => Token::PuncOr,
            },

            b'^' => match self.peek() {
                Some(b'=') => {
                    unsafe { self.advance_unchecked() };
                    Token::PuncXorEq
                }
                _ => Token::PuncXor,
            },

            c if lexer_impls::numbers::is_valid_digit(c) => {
                // SAFETY: self.index is always 1 character ahead of self.start due
                // to fixed advance unchecked
                match unsafe { self.lex_ambiguous_number_literal() } {
                    Ok(tok) => tok,
                    Err(e) => return Err(e),
                }
            }

            c if lexer_impls::identifiers::is_valid_identifier_head(c) => {
                // SAFETY: self.index is always 1 character ahead of self.start due
                // to fixed advance unchecked, and character validity is determined by
                // `is_valid_identifier_head`
                unsafe { self.lex_identifier() }
            }

            // always invalid characters:
            //
            // - anything up until the " " character (byte 0x20, decimal 32)
            //   except whitespace such as "\t", "\r", "\n"
            // - "#"
            // - "$"
            // - "?"
            // - "@"
            // - "\" outside of a string escape
            // - "`"
            // - anything outside of the ascii range (outside of strings)
            //   or in other words, values higher than [DEL] (byte 0x7f, decimal 127)
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

    /// # Safety
    ///
    /// more of a correctness requirement: use `extract_literal` instead, or
    /// otherwise the next call to `extract_literal` will duplicate your literals.
    #[inline]
    pub const unsafe fn extract_literal_copy(&self) -> LexerResult<&'source [u8]> {
        match self.literal {
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

    #[inline]
    pub fn get_lexer_debug_state(&self) -> String {
        let (line, column) = self.get_line_column();
        let start = self.start();
        let index = self.index();
        // its fine to duplicate here because its just debug string
        let lit = unsafe { self.extract_literal_copy() };
        format!(
            "lexer error at {}:{} (index {}..{}), possible literal: {:?}",
            line, column, start, index, lit
        )
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

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, LexerError, LexerResult},
        source_code::SourceCode,
        types::Token,
    };

    #[test]
    fn higher_level_api_test() {
        let text = "     \n\tlet freeform() ; = <= + 3 >= != \n";
        let mut lexer = Lexer::new(SourceCode::new(text));
        assert_eq!(lexer.next(), Some(Token::KwLet));
        assert_eq!(lexer.extract_literal(), Err(LexerError::NoLiteralToExtract));

        assert_eq!(lexer.next(), Some(Token::LitIdentifier));
        assert_eq!(lexer.extract_literal(), Ok(&b"freeform"[..]));
        assert_eq!(lexer.extract_literal(), Err(LexerError::NoLiteralToExtract));

        assert_eq!(lexer.next(), Some(Token::IndentLParen));
    }

    #[test]
    fn invalid_characters_test() {
        // [0..=255]
        let bytes = const {
            let mut array = [0u8; 256];
            let mut i = 0;
            while i < 256 {
                array[i] = i as u8;
                i += 1;
            }
            array
        };
        let lossy_str = String::from_utf8_lossy(&bytes);
        let mut l = Lexer::new(SourceCode::new(&lossy_str));
        let mut pairs = vec![];

        while !l.is_at_end() {
            let res = l.clone().lex_single_token() == Err(LexerError::InvalidCharacter);
            let byte = lossy_str.as_bytes()[l.index()];
            l.advance();
            pairs.push((res, byte));
        }

        // for item in pairs {
        //     println!("{}: \"{}\", byte {}", if item.0 { "invalid" } else { "valid" }, item.1 as char, item.1);
        // }
    }

    #[test]
    fn test_operators() {
        let source = "! - * / + << >> < <= > >= == != = += -= *= /= %= &= |= ^= <<= >>=";
        let mut l = Lexer::new(SourceCode::new(source));

        let expected = [
            Token::PuncBang,
            Token::PuncMinus,
            Token::PuncStar,
            Token::PuncSlash,
            Token::PuncPlus,
            Token::PuncShl,
            Token::PuncShr,
            Token::PuncLt,
            Token::PuncLtEq,
            Token::PuncGt,
            Token::PuncGtEq,
            Token::PuncEqEq,
            Token::PuncBangEq,
            Token::PuncEq,
            Token::PuncPlusEq,
            Token::PuncMinusEq,
            Token::PuncStarEq,
            Token::PuncSlashEq,
            Token::PuncModuloEq,
            Token::PuncAndEq,
            Token::PuncOrEq,
            Token::PuncXorEq,
            Token::PuncShlEq,
            Token::PuncShrEq,
        ];
        let mut index = 0;

        let mut val: LexerResult<Token>;
        loop {
            val = l.lex_single_token();
            match val {
                Ok(tok) => {
                    assert_eq!(expected[index], tok, "Error at {}th, expected {:?}", index, expected[index]);
                    println!("Matched {:?} with {:?}", tok, expected[index]);
                    index += 1;
                }
                Err(LexerError::Eof) => break,
                Err(e) => {
                    panic!("lexer error: {:?}\n\t{}", e, l.get_lexer_debug_state());
                }
            }
        }
    }

    // todo: test {token}EOF for all tokens (FOR UB)
    #[test]
    fn eof_after_all_tokens_no_ub_for_miri() {
        let fail_sources = &[
            "2485.", "\"fdf", "\"", "'v", "'", r#""\""#, r#""\"#, r#""\m""#, r#""\\"#, r#"'\'"#, r#"'\\"#, r#"'\"#, r#"'\m'"#,
        ];
        let sources = &[
            // ident, eof
            "let",
            "fn",
            "return",
            "extern",
            "const",
            "mut",
            "anymut",
            "compiletime",
            "runtime",
            "static",
            "type",
            "cast",
            "struct",
            "enum",
            "union",
            "true",
            "false",
            "uninit",
            // {self}, ident
            "48545",
            "2485.1",
            "\"fdf\"",
            "\"\\\\\"",
            "\"\\0\"",
            "'v'",
            "_",
            "a",
            "_1",
            "_a",
            "__",
            ".",
            ",",
            ";",
            ":",
            "->",
            "=",
            "==",
            "!",
            "!=",
            "<",
            "<=",
            ">",
            ">=",
            "+",
            "-",
            "*",
            "/",
            "%",
            "&",
            "|",
            "^",
            "<<",
            ">>",
            "+=",
            "-=",
            "*=",
            "/=",
            "%=",
            "&=",
            "|=",
            "^=",
            "<<=",
            ">>=",
            "(",
            ")",
            "{",
            "}",
            "[",
            "]",
        ];

        for correct in sources {
            let mut l = Lexer::new(SourceCode::new(correct));
            assert!(!l.is_at_end());
            assert!(l.lex_single_token().is_ok());
            assert!(l.is_at_end());
            let index = l.index();
            assert_eq!(l.lex_single_token(), Err(LexerError::Eof));
            assert!(l.is_at_end());
            assert_eq!(l.index(), index);

            let mut new_source = correct.to_string();
            new_source.push('.');
            let mut l = Lexer::new(SourceCode::new(&new_source));
            assert!(!l.is_at_end());
            let first = l.lex_single_token();
            if first.is_ok() {
                assert!(!l.is_at_end(), "source: \"{}\", {:?}", &new_source, l.get_lexer_debug_state());
            }
            let second = l.lex_single_token();
            // we just checked
            match first {
                Ok(_) => {
                    assert_eq!(second, Ok(Token::PuncDot));
                }
                Err(_) => {
                    assert_eq!(second, Err(LexerError::Eof));
                }
            }
            assert!(l.is_at_end());

            let mut new_source = correct.to_string();
            new_source.push('f');
            let mut l = Lexer::new(SourceCode::new(&new_source));
            assert!(!l.is_at_end());
            let first = l.lex_single_token();
            assert!(first.is_ok());
            match first.unwrap() {
                Token::LitIdentifier => {
                    assert!(l.is_at_end());
                }
                Token::LitInteger => {
                    let lit = l.extract_literal().unwrap();
                    assert_eq!(lit, &b"48545"[..], "source: \"{}\", {:?}", &new_source, l.get_lexer_debug_state());
                }
                _ => {
                    assert!(!l.is_at_end());
                }
            }
            let second = l.lex_single_token();
            match first.unwrap() {
                Token::LitIdentifier => {
                    assert_eq!(second, Err(LexerError::Eof));
                }
                _ => {
                    assert_eq!(second, Ok(Token::LitIdentifier))
                }
            }
            assert!(l.is_at_end());
        }

        for incorrect in fail_sources {
            let mut l = Lexer::new(SourceCode::new(incorrect));
            assert!(!l.is_at_end());
            assert!(l.lex_single_token().is_err());
            assert!(l.is_at_end(), "source: \"{}\", {:?}", &incorrect, l.get_lexer_debug_state());
            let index = l.index();
            assert_eq!(l.lex_single_token(), Err(LexerError::Eof));
            assert!(l.is_at_end());
            assert_eq!(l.index(), index);
        }
    }
}
