use crate::lexer::Lexer;
use core::hint::assert_unchecked;
use core::slice;

pub mod high_level;
pub mod identifiers;
pub mod numbers;
pub mod skip_whitespace;

/// byte-level traversal
impl<'source> Lexer<'source> {
    #[inline(always)]
    pub const fn is_at_end(&self) -> bool {
        self.index >= self.source.len()
    }

    /// # Safety
    ///
    /// `self.is_at_end()` must be false.
    #[inline]
    #[track_caller]
    pub const unsafe fn peek_unchecked(&self) -> u8 {
        unsafe {
            assert_unchecked(!self.is_at_end());
            *self.source.as_bytes().as_ptr().add(self.index)
        }
    }

    #[inline]
    #[track_caller]
    pub const fn peek(&self) -> Option<u8> {
        if self.is_at_end() {
            None
        } else {
            Some(unsafe { self.peek_unchecked() })
        }
    }

    #[inline]
    #[track_caller]
    pub const fn peek_default(&self) -> u8 {
        if self.is_at_end() { 0 } else { unsafe { self.peek_unchecked() } }
    }

    /// After this function returns, you may be at the end.
    ///
    /// # Safety
    ///
    /// `self.is_at_end()` must be false.
    #[inline]
    #[track_caller]
    pub const unsafe fn advance_unchecked(&mut self) -> u8 {
        unsafe {
            let byte = self.peek_unchecked();
            self.index += 1;
            if byte == b'\n' {
                self.line += 1;
                self.column = 1;
                byte
            } else {
                self.column += 1;
                byte
            }
        }
    }

    /// After this function returns, you may be at the end.
    #[inline]
    #[track_caller]
    pub const fn advance(&mut self) -> Option<u8> {
        if self.is_at_end() {
            None
        } else {
            Some(unsafe { self.advance_unchecked() })
        }
    }

    /// After this function returns, you may be at the end.
    #[inline]
    #[track_caller]
    pub const fn advance_default(&mut self) -> u8 {
        if self.is_at_end() { 0 } else { unsafe { self.advance_unchecked() } }
    }

    /// # Safety
    ///
    /// `self.is_at_end()` must be false.
    #[inline]
    #[track_caller]
    pub const unsafe fn peek_next_unchecked(&self) -> u8 {
        unsafe {
            assert_unchecked(self.index + 1 < self.source.len());
            *self.source.as_bytes().as_ptr().add(self.index + 1)
        }
    }

    #[inline]
    #[track_caller]
    pub const fn peek_next(&self) -> Option<u8> {
        if self.index + 1 >= self.source.len() {
            None
        } else {
            Some(unsafe { self.peek_next_unchecked() })
        }
    }

    #[inline]
    #[track_caller]
    pub const fn peek_next_default(&self) -> u8 {
        if self.index + 1 >= self.source.len() {
            0
        } else {
            unsafe { self.peek_next_unchecked() }
        }
    }

    /// After this function returns, you may be at the end.
    ///
    /// # Safety
    ///
    /// `self.is_at_end()` must be false.
    #[inline]
    #[track_caller]
    pub const unsafe fn matches_unchecked(&mut self, expected: u8) -> bool {
        unsafe {
            assert_unchecked(!self.is_at_end());
            let byte = self.peek_unchecked();
            if byte == expected {
                self.advance_unchecked();
                true
            } else {
                false
            }
        }
    }

    /// After this function returns, you may be at the end.
    #[inline]
    #[track_caller]
    pub const fn matches(&mut self, expected: u8) -> Option<bool> {
        let Some(byte) = self.peek() else {
            return None;
        };

        if byte == expected {
            self.index += 1;
            Some(true)
        } else {
            Some(false)
        }
    }

    /// After this function returns, you may be at the end.
    #[inline]
    #[track_caller]
    pub const fn matches_default(&mut self, expected: u8) -> bool {
        let Some(byte) = self.peek() else {
            return false;
        };

        if byte == expected {
            self.index += 1;
            true
        } else {
            false
        }
    }

    #[inline]
    #[track_caller]
    pub const fn matches_bytes(&mut self, expected: &[u8]) -> bool {
        let mut index = 0;
        while !self.is_at_end() && index < expected.len() {
            let Some(byte) = self.peek() else {
                return false;
            };

            // bounds check hopefully optimized away by the loop condition
            if byte != expected[index] {
                return false;
            }

            unsafe { self.advance_unchecked() };
            index += 1;
        }
        true
    }

    /// # Safety
    ///
    /// - the entirety of the range `self.source.as_bytes[self.start..self.index]`
    ///   must be in bounds.
    ///
    /// NOTE: `self.index` may equal `self.source.len()` and does not pose a problem.
    #[inline]
    #[track_caller]
    pub const unsafe fn slice_here(&self) -> &'source [u8] {
        unsafe {
            let ptr = self.source.as_bytes().as_ptr().add(self.start);
            let len = self.index - self.start;
            slice::from_raw_parts(ptr, len)
        }
    }

    /// # Safety
    ///
    /// - `self.index` must be bigger than 0
    /// - `self.index` must be smaller than or equal to self.source.len()
    /// - `self.line` must be bigger than 0
    #[inline]
    #[track_caller]
    pub const unsafe fn backtrack_unchecked(&mut self) -> u8 {
        unsafe {
            self.index = self.index.unchecked_sub(1);
            let byte = self.peek_unchecked();
            if byte == b'\n' {
                self.line = self.line.unchecked_sub(1);
                // TODO DANGER WE HAVE TO BACK TRACK UNTIL THE PREVIOUS NEWLINE OR START OF SOURCE
                // AND FIX UP self.column FOR THE FUCKING DEBUSF GHBKJL;FSDLGSDL;G
                self.column = 1;
            }
            byte
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, LexerError},
        source_code::SourceCode,
        types::Token,
    };

    #[test]
    fn subsequent_peek_and_advance_never_none() {
        let source = "hi";
        let mut lexer = Lexer::new(SourceCode::new(source));

        assert!(!lexer.is_at_end());
        assert_eq!(lexer.peek(), Some(b'h'));
        assert_eq!(lexer.advance(), Some(b'h'));

        assert!(!lexer.is_at_end());
        assert_eq!(lexer.peek(), Some(b'i'));
        assert_eq!(lexer.advance(), Some(b'i'));

        assert!(lexer.is_at_end());

        let index = lexer.index();

        assert_eq!(lexer.peek(), None);
        assert_eq!(lexer.advance(), None);

        assert_eq!(lexer.index(), index);
    }

    #[test]
    fn lexer_accessors_work() {
        let mut lexer = Lexer::new(SourceCode::new("let x = 42;"));
        assert_eq!(lexer.index(), lexer.index);
        assert_eq!(lexer.index(), 0);
        assert_eq!(lexer.start(), lexer.start);
        assert_eq!(lexer.start(), 0);
        assert_eq!(lexer.get_line_column(), (1, 0));

        assert_eq!(lexer.next(), Some(Token::KwLet));
        assert_eq!(lexer.start(), 0);
        assert_eq!(lexer.index(), 3);

        assert_eq!(lexer.next(), Some(Token::LitIdentifier));
        assert_eq!(lexer.start(), 4);
        assert_eq!(lexer.index(), 5);
        assert_eq!(lexer.extract_literal(), Ok(&b"x"[..]));
    }

    #[test]
    fn bytelevel_peek() {
        let source = "hi";
        let mut lexer = Lexer::new(SourceCode::new(source));

        assert!(!lexer.is_at_end());

        assert_eq!(lexer.peek(), Some(b'h'));
        assert_eq!(lexer.peek_default(), b'h');
        // SAFETY: within bounds
        assert_eq!(unsafe { lexer.peek_unchecked() }, b'h');

        assert_eq!(lexer.clone().advance(), Some(b'h'));
        assert_eq!(lexer.clone().advance_default(), b'h');
        // SAFETY: within bounds
        assert_eq!(unsafe { lexer.clone().advance_unchecked() }, b'h');

        {
            let mut lexer = lexer.clone();
            assert_eq!(lexer.matches(b'h'), Some(true));
            assert_eq!(lexer.index(), 1);
        };
        {
            let mut lexer = lexer.clone();
            assert_eq!(lexer.matches(b'x'), Some(false));
            assert_eq!(lexer.index(), 0);
        };
        {
            let mut lexer = lexer.clone();
            assert!(lexer.matches_default(b'h'));
            assert_eq!(lexer.index(), 1);
        };
        {
            let mut lexer = lexer.clone();
            assert!(!lexer.matches_default(b'x'));
            assert_eq!(lexer.index(), 0);
        };

        lexer.advance();

        assert_eq!(lexer.index(), 1);

        assert_eq!(lexer.peek(), Some(b'i'));
        assert_eq!(lexer.peek_default(), b'i');
        // SAFETY: within bounds
        assert_eq!(unsafe { lexer.peek_unchecked() }, b'i');

        lexer.advance();

        assert_eq!(lexer.index(), 2);

        assert_eq!(lexer.peek(), None);
        assert_eq!(lexer.peek_default(), 0);

        assert_eq!(lexer.advance(), None);
        assert_eq!(lexer.advance_default(), 0);

        assert!(lexer.is_at_end());

        // SAFETY: self.index == self.source.len() does not pose a problem as per slice_here docs
        let slice = unsafe { lexer.slice_here() };
        assert_eq!(slice, b"hi");
    }

    #[test]
    fn lexes_strings_anc_charlits() {
        let text = r#"
            "10 string \" ends here ->"
            'V'
        "#;
        let mut lexer = Lexer::new(SourceCode::new(text));
        assert_eq!(lexer.lex_single_token(), Ok(Token::LitStr));
        assert_eq!(lexer.extract_literal(), Ok(&b"10 string \\\" ends here ->"[..]));
        assert_eq!(lexer.lex_single_token(), Ok(Token::LitChar));
        assert_eq!(lexer.extract_literal(), Ok(&b"V"[..]));

        let fail1 = r#"
            "unterminated
        "#;
        let mut lexer = Lexer::new(SourceCode::new(fail1));
        assert_eq!(lexer.lex_single_token(), Err(LexerError::UnexpectedEofWhile(Token::LitStr)));
        assert_eq!(lexer.extract_literal(), Err(LexerError::NoLiteralToExtract));

        let fail2 = r#"
            'v
        "#;
        let mut lexer = Lexer::new(SourceCode::new(fail2));
        assert_eq!(lexer.lex_single_token(), Err(LexerError::InvalidCharacter));
        assert_eq!(lexer.extract_literal(), Err(LexerError::NoLiteralToExtract));
    }
}
