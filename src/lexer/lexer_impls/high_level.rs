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

    /// if this function returns a value matching `Ok(t) if t.is_identifier_extractable()`,
    /// you can extract the specific literal by using `self.extract_literal()` and
    /// unsafely unwrap it **once** before any modification.
    ///
    /// # Safety
    ///
    /// - `self.start` points to the first character of the identifier
    /// - `self.index` points to one character after `self.start` (may be at the end)
    /// - character pointed to by `self.start` is `alnum | "_"`
    ///
    /// After this function returns, you may be at the end.
    pub const unsafe fn parse_identifier(&mut self) -> LexerResult<Token> {
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
        self.literal = Some(slice);

        Ok(unsafe { check_identifier_actual_token(self, slice) })
    }

    /// if this function returns a value matching `Ok(t) if t.is_identifier_extractable()`,
    /// you can extract the specific literal by using `self.extract_literal()` and
    /// unsafely unwrap it **once** before any modification.
    ///
    /// # Safety
    ///
    /// - `self.start` points to the first character of the identifier
    /// - `self.index` points to one character after `self.start` (may be at the end)
    /// - character pointed to by `self.start` is `"`.
    ///
    /// After this function returns, you may be at the end.
    #[inline]
    pub const unsafe fn parse_quoted_string(&mut self) -> LexerResult<Token> {
        if self.is_at_end() {
            return Err(LexerError::UnexpectedEofWhile(Token::LitStr));
        }

        while !self.is_at_end() {
            // SAFETY: we are guaranteed to not be at the end here

            let byte = unsafe { self.peek_unchecked() };

            match byte {
                b'"' => break,
                b'\\' => {
                    let Some(escaped) = self.peek_next() else {
                        return Err(LexerError::UnexpectedEofWhile(Token::LitStr));
                    };

                    match escaped {
                        b'"' | b't' | b'n' | b'r' | b'\\' | b'0' => {
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
                        _ => {
                            // invalid escape
                            return Err(LexerError::InvalidEscapeSequence);
                        }
                    }
                }
                _ => {
                    // allowed character
                    unsafe { self.advance_unchecked() };
                }
            }
        }

        if self.is_at_end() {
            return Err(LexerError::UnexpectedEofWhile(Token::LitStr));
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
    pub const unsafe fn parse_character_literal(&mut self) -> LexerResult<Token> {
        if self.is_at_end() {
            return Err(LexerError::UnexpectedEofWhile(Token::LitChar));
        }

        let byte = unsafe { self.peek_unchecked() };

        match byte {
            b'\\' => {
                let Some(escaped) = self.peek_next() else {
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
                    _ => {
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
    pub const unsafe fn parse_ambiguous_number_literal(&mut self) -> LexerResult<Token> {
        while !self.is_at_end() {
            let byte = unsafe { self.peek_unchecked() };

            match byte {
                c if lexer_impls::numbers::is_valid_digit(c) => unsafe { self.advance_unchecked() },
                b'.' => {
                    unsafe {
                        self.advance_unchecked();
                        return parse_partial_float(self);
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
pub const unsafe fn parse_partial_float(lexer: &mut Lexer<'_>) -> LexerResult<Token> {
    if lexer.is_at_end() {
        return Err(LexerError::UnexpectedEofWhile(Token::LitFloat));
    }

    while !lexer.is_at_end() {
        let byte = unsafe { lexer.peek_unchecked() };

        match byte {
            c if lexer_impls::numbers::is_valid_digit(c) => unsafe { lexer.advance_unchecked() },
            _ => {
                break;
            }
        };
    }

    // SAFETY: self.start is 1 after the start quote, self.index is at the end quote
    // self.index can at most equal the source length here, and that is fine
    let slice = unsafe { lexer.slice_here() };

    lexer.literal = Some(slice);

    Ok(Token::LitFloat)
}
