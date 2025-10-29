use crate::lexer::Lexer;
use crate::lexer::LexerError;
use crate::lexer::LexerResult;
use crate::lexer::lexer_impls;
use crate::lexer::lexer_impls::identifiers::check_identifier_actual_token;
use crate::lexer::lexer_impls::identifiers::is_valid_identifier_tail;
use crate::lexer::lexer_impls::skip_whitespace::skip_whitespace_impl;
use crate::types::Token;

/// higher level lexers
impl<'source> Lexer<'source> {
    /// After this function returns, you may be at the end.
    #[inline]
    pub const fn skip_whitespace(&mut self) {
        skip_whitespace_impl(self);
    }

    /// if this function returns a value matching `t if t.is_identifier_extractable()`,
    /// you can extract the specific literal by using `self.extract_literal()` and
    /// unsafely unwrap it **once** before any modification to the lexer.
    ///
    /// # Safety
    ///
    /// - `self.start` points to the first character of the identifier
    /// - `self.index` points to one character after `self.start` (may be at the end)
    /// - character pointed to by `self.start` is `alnum | "_"`
    ///
    /// After this function returns, you may be at the end.
    pub const unsafe fn lex_identifier(&mut self) -> Token {
        while !self.is_at_end() {
            // SAFETY: we are guaranteed to not be at the end here

            let byte = unsafe { self.peek_unchecked() };
            if is_valid_identifier_tail(byte) {
                unsafe { self.advance_unchecked() };
            } else {
                break;
            }
        }

        // SAFETY: self.index can at most equal the source length here, and that is fine
        let slice = unsafe { self.slice_here() };

        // SAFETY: caller ensures self.start and self.index is at least 1 character apart
        let res = unsafe { check_identifier_actual_token(self, slice) };
        if res.is_identifier_extractable() {
            self.literal = Some(slice);
        }
        res
    }

    /// if this function returns a value matching `Ok(t) if t.is_identifier_extractable()`,
    /// you can extract the specific literal by using `self.extract_literal()` and
    /// unsafely unwrap it **once** before any modification.
    ///
    /// # Safety
    ///
    /// - `self.start` points to the first quote
    /// - `self.index` points to at least one character after `self.start` but within the string (may be at the end)
    /// - character pointed to by `self.start` is `"`.
    ///
    /// After this function returns, you may be at the end.
    #[inline]
    pub const unsafe fn lex_quoted_string(&mut self) -> LexerResult<Token> {
        if self.is_at_end() {
            return Err(LexerError::UnexpectedEofWhile(Token::LitStr));
        }

        while !self.is_at_end() {
            // SAFETY: we are guaranteed to not be at the end here

            let byte = unsafe { self.advance_unchecked() };

            match byte {
                b'"' => {
                    unsafe { self.backtrack_unchecked() };
                    break;
                }
                b'\\' => {
                    let Some(escaped) = self.advance() else {
                        return Err(LexerError::UnexpectedEofWhile(Token::LitStr));
                    };

                    match escaped {
                        b'"' | b't' | b'n' | b'r' | b'\\' | b'0' => {
                            // allow escape
                            continue;
                        }
                        b'x' => {
                            // byte escape sequence
                            // follow rust: \xNN where n is a hexadecimal character, not shorter, not longer.
                            return Err(LexerError::WithMessage("byte escape sequences are not implemented yet"));
                        }
                        _ => {
                            // invalid escape
                            if self.is_at_end() {
                                return Err(LexerError::UnexpectedEofWhile(Token::LitStr));
                            }
                            // "hello world \m\m\" "
                            //                     ^
                            while !self.is_at_end() {
                                let byte = unsafe { self.advance_unchecked() };
                                match byte {
                                    b'"' => {
                                        unsafe { self.backtrack_unchecked() };
                                        break;
                                    }
                                    b'\\' => {
                                        let Some(_) = self.advance() else {
                                            return Err(LexerError::UnexpectedEofWhile(Token::LitStr));
                                        };
                                    }
                                    _ => continue,
                                }
                            }
                            if self.is_at_end() {
                                return Err(LexerError::UnexpectedEofWhile(Token::LitStr));
                            }
                            // the loop exit conditions were to either be at the end, or be at a quote
                            // we checked if we were at the end above, so we're guaranteed to be at a quote.
                            unsafe { self.advance_unchecked() };

                            return Err(LexerError::InvalidEscapeSequence);
                        }
                    }
                }
                _ => {
                    // allowed character
                    continue;
                }
            }
        }

        if self.is_at_end() {
            return Err(LexerError::UnexpectedEofWhile(Token::LitStr));
        }

        unsafe {
            if self.peek_unchecked() != b'"' {
                self.advance_unchecked();
                return Err(LexerError::InvalidCharacter);
            }
        }

        // self.index guaranteed pointing to `"`

        // skip the first quote character
        self.start += 1;

        // SAFETY: self.start is 1 after the start quote, self.index is at the end quote
        // self.index is guaranteed lesser than the source length here
        let slice = unsafe { self.slice_here() };

        // consume the end quote
        unsafe {
            self.advance_unchecked();
        }

        self.literal = Some(slice);

        Ok(Token::LitStr)
    }

    /// if this function returns a value matching `Ok(t) if t.is_identifier_extractable()`,
    /// you can extract the specific literal by using `self.extract_literal()` and
    /// unsafely unwrap it **once** before any modification.
    ///
    /// # Safety
    ///
    /// - `self.start` points to the first character of the identifier
    /// - `self.index` points to one character after `self.start` (may be at the end)
    /// - character pointed to by `self.start` is `'`.
    ///
    /// After this function returns, you may be at the end.
    #[inline]
    pub const unsafe fn lex_character_literal(&mut self) -> LexerResult<Token> {
        if self.is_at_end() {
            return Err(LexerError::UnexpectedEofWhile(Token::LitChar));
        }

        let byte = unsafe { self.peek_unchecked() };

        match byte {
            b'\\' => {
                let Some(escaped) = self.peek_next() else {
                    unsafe { self.advance_unchecked() };
                    return Err(LexerError::UnexpectedEofWhile(Token::LitChar));
                };

                match escaped {
                    b'\'' | b't' | b'n' | b'r' | b'\\' | b'0' => {
                        // allow escape and advance twice
                        unsafe {
                            self.advance_unchecked();
                            self.advance_unchecked();
                        };
                    }
                    b'x' => {
                        // byte escape sequence
                        // follow rust: \xNN where n is a hexadecimal character, not shorter, not longer.
                        return Err(LexerError::WithMessage("byte escape sequences are not implemented yet"));
                    }
                    // '\mf;
                    //    ^
                    _ => {
                        unsafe {
                            self.advance_unchecked();
                            self.advance_unchecked();
                        }
                        if self.is_at_end() {
                            return Err(LexerError::UnexpectedEofWhile(Token::LitChar));
                        }

                        let val = unsafe { self.advance_unchecked() };
                        if val != b'\'' {
                            return Err(LexerError::UnclosedCharLiteral);
                        }

                        // invalid escape
                        return Err(LexerError::InvalidEscapeSequence);
                    }
                }
            }
            _ => unsafe {
                self.advance_unchecked();
            },
        };

        if self.is_at_end() {
            return Err(LexerError::UnexpectedEofWhile(Token::LitChar));
        }

        unsafe {
            if self.peek_unchecked() != b'\'' {
                self.advance_unchecked();
                return Err(LexerError::InvalidCharacter);
            }
        }

        // self.index guaranteed pointing to `'`

        // skip the first quote character
        self.start += 1;

        // SAFETY: self.start is 1 after the start quote, self.index is at the end quote
        // self.index is guaranteed lesser than the source length here
        let slice = unsafe { self.slice_here() };

        // consume the end quote
        unsafe {
            self.advance_unchecked();
        }

        self.literal = Some(slice);

        Ok(Token::LitChar)
    }

    /// if this function returns a value matching `Ok(t) if t.is_identifier_extractable()`,
    /// you can extract the specific literal by using `self.extract_literal()` and
    /// unsafely unwrap it **once** before any modification.
    ///
    /// # Safety
    ///
    /// - `self.start` points to the first character of the identifier
    /// - `self.index` points to one character after `self.start` (may be at the end)
    /// - character pointed to by `self.start` passes `lexer_impls::numbers::is_valid_digit`.
    ///
    /// After this function returns, you may be at the end.
    #[inline]
    pub const unsafe fn lex_ambiguous_number_literal(&mut self) -> LexerResult<Token> {
        while !self.is_at_end() {
            // SAFETY: we are guaranteed to not be at the end here

            let byte = unsafe { self.peek_unchecked() };

            match byte {
                c if lexer_impls::numbers::is_valid_digit(c) => unsafe { self.advance_unchecked() },
                b'.' => {
                    unsafe {
                        self.advance_unchecked();
                        return lex_dot_after_integer(self);
                    };
                }
                _ => {
                    break;
                }
            };
        }

        // SAFETY: self.start is 1 after the start quote, self.index is at the end quote
        // self.index can at most equal the source length here, and that is fine
        let slice = unsafe { self.slice_here() };

        self.literal = Some(slice);

        Ok(Token::LitInteger)
    }
}

/// # Safety
///
/// - `lexer.source.as_bytes()[lexer.start..lexer.index - 1]` must be a slice where all elements
/// - pass `lexer_impls::numbers::is_valid_digit`.
/// - `lexer.source.as_bytes()[lexer.index - 1]` must be a `.` character. (you should've already consumed the dot)
#[inline]
pub const unsafe fn lex_dot_after_integer(lexer: &mut Lexer<'_>) -> LexerResult<Token> {
    if lexer.is_at_end() {
        // TODO:
        // @backtracking:1 = return the lit integer and set index properly for dot
        return Err(LexerError::UnexpectedEofWhile(Token::LitFloat));
    }

    // checking part after dot
    // 100.53
    // 100.sum()
    // 100.!
    //     ^
    let peeked = unsafe { lexer.peek_unchecked() };
    match peeked {
        // valid float literal
        c if lexer_impls::numbers::is_valid_digit(c) => {
            // consume the first digit of the decimal part
            unsafe { lexer.advance_unchecked() };

            // keep lexing digits, if any
            while !lexer.is_at_end() {
                // SAFETY: we are guaranteed to not be at the end here

                let byte = unsafe { lexer.peek_unchecked() };

                match byte {
                    c if lexer_impls::numbers::is_valid_digit(c) => unsafe { lexer.advance_unchecked() },
                    // method calls on floats are unambiguously lexed
                    _ => {
                        break;
                    }
                };
            }
        }
        // 10. abs()
        // TODO: allow spaces after the dot and expect an identifier head, then parse identifier
        c if lexer_impls::skip_whitespace::is_whitespace(c) => {
            // lexer state:
            //      10. abs()
            //         ^ known whitespace (comments not allowed here) TODO support comments here
            // @backtracking:2 = return the lit integer and set index properly for dot
            return Err(LexerError::WithMessage("lexing whitespace after `{integer}.` is todo"));
        }
        // 10.abs()
        c if lexer_impls::identifiers::is_valid_identifier_head(c) => {
            // lexer state:
            //      10.abs()
            //         ^ known identifier head
            // @backtracking:3 = return the lit integer and set index properly for the dot
            //
            // TODO method call on integer literal
            return Err(LexerError::WithMessage("lexing method calls on integer literals are todo"));
        }
        _ => return Err(LexerError::UnexpectedEofWhile(Token::LitFloat)),
    }

    // SAFETY: self.start is 1 after the start quote, self.index is at the end quote
    // self.index can at most equal the source length here, and that is fine
    let slice = unsafe { lexer.slice_here() };

    lexer.literal = Some(slice);

    Ok(Token::LitFloat)
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, LexerError, lexer_impls},
        source_code::SourceCode,
        types::Token,
    };

    #[test]
    fn lexes_identifier_correctly() {
        let expected = [Token::KwConst, Token::LitIdentifier, Token::KwType, Token::LitIdentifier];

        ["const", "_my_struct", "type", "conster"]
            .into_iter()
            .map(|s| {
                let mut l = Lexer::new(SourceCode::new(s));
                l.advance();
                // SAFETY: as `lex_identifier` describes,
                // - self.start points to the first character
                // - self.index points to one character after self.start
                // - character pointed to by self.start is alnum | "_"
                unsafe { l.lex_identifier() }
            })
            .zip(expected)
            .for_each(|(got, expected)| {
                assert_eq!(got, expected);
            });
    }

    #[test]
    fn lexes_number_literals() {
        let source = "927364";
        let mut lexer = Lexer::new(SourceCode::new(source));
        assert!(matches!(
            lexer.advance(),
            Some(c) if lexer_impls::numbers::is_valid_digit(c)
        ));

        // SAFETY: as `lex_identifier` describes,
        // - self.start points to the first character
        // - self.index points to one character after self.start
        // - character pointed to by self.start passes `is_valid_digit`
        assert_eq!(unsafe { lexer.lex_ambiguous_number_literal() }, Ok(Token::LitInteger));
        assert_eq!(lexer.extract_literal(), Ok(&b"927364"[..]));

        let source = "10.3";
        let mut lexer = Lexer::new(SourceCode::new(source));
        assert!(matches!(
            lexer.advance(),
            Some(c) if lexer_impls::numbers::is_valid_digit(c)
        ));

        // SAFETY: as `lex_identifier` describes,
        // - self.start points to the first character
        // - self.index points to one character after self.start
        // - character pointed to by self.start passes `is_valid_digit`
        assert_eq!(unsafe { lexer.lex_ambiguous_number_literal() }, Ok(Token::LitFloat));
        assert_eq!(lexer.extract_literal(), Ok(&b"10.3"[..]));

        let invalid = "10.sdf";
        let mut lexer = Lexer::new(SourceCode::new(invalid));
        assert!(matches!(
            lexer.advance(),
            Some(c) if lexer_impls::numbers::is_valid_digit(c)
        ));

        assert!(lexer.lex_single_token().is_err());
        assert_eq!(lexer.extract_literal(), Err(LexerError::NoLiteralToExtract));
    }

    #[test]
    fn litchar_extensive() {
        let text = "'\\mf";
        let mut l = Lexer::new(SourceCode::new(text));
        assert_eq!(l.lex_single_token(), Err(LexerError::UnclosedCharLiteral));
        assert!(l.is_at_end());

        let text = "'\\m'";
        let mut l = Lexer::new(SourceCode::new(text));
        assert_eq!(l.lex_single_token(), Err(LexerError::InvalidEscapeSequence));
        assert!(l.is_at_end());

        let text = "'\\m";
        let mut l = Lexer::new(SourceCode::new(text));
        assert_eq!(l.lex_single_token(), Err(LexerError::UnexpectedEofWhile(Token::LitChar)));
        assert!(l.is_at_end());
    }

    #[test]
    fn quoted_string_invalid_invalid() {
        let text = r#"
        "\m\m""#;
        let mut l = Lexer::new(SourceCode::new(text));
        assert_eq!(l.lex_single_token(), Err(LexerError::InvalidEscapeSequence));
        assert!(l.is_at_end());
        let text = r#"
        "\m\"   ""#;
        let mut l = Lexer::new(SourceCode::new(text));
        assert_eq!(l.lex_single_token(), Err(LexerError::InvalidEscapeSequence));
        assert!(l.is_at_end());
        let text = r#""\m\n"#;
        let mut l = Lexer::new(SourceCode::new(text));
        assert_eq!(l.lex_single_token(), Err(LexerError::UnexpectedEofWhile(Token::LitStr)));
        assert!(l.is_at_end(), "source: `{}`, lexer:\n\t{}", text, l.get_lexer_debug_state());
    }
}
