use crate::lexer::Lexer;
use core::hint::assert_unchecked;
use core::slice;

pub mod high_level;
pub mod identifiers;
pub mod skip_whitespace;
pub mod numbers;

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
    pub const unsafe fn peek_unchecked(&self) -> u8 {
        unsafe {
            assert_unchecked(!self.is_at_end());
            *self.source.as_bytes().as_ptr().add(self.index)
        }
    }

    #[inline]
    pub const fn peek(&self) -> Option<u8> {
        if self.is_at_end() {
            None
        } else {
            Some(unsafe { self.peek_unchecked() })
        }
    }

    #[inline]
    pub const fn peek_default(&self) -> u8 {
        if self.is_at_end() { 0 } else { unsafe { self.peek_unchecked() } }
    }

    /// After this function returns, you may be at the end.
    ///
    /// # Safety
    ///
    /// `self.is_at_end()` must be false.
    #[inline]
    pub const unsafe fn advance_unchecked(&mut self) -> u8 {
        unsafe {
            let byte = self.peek_unchecked();
            self.index += 1;
            if byte == b'\n' {
                self.line += 1;
                self.column = 0;
            }
            self.column += 1;
            byte
        }
    }

    /// After this function returns, you may be at the end.
    #[inline]
    pub const fn advance(&mut self) -> Option<u8> {
        if self.is_at_end() {
            None
        } else {
            Some(unsafe { self.advance_unchecked() })
        }
    }

    /// After this function returns, you may be at the end.
    #[inline]
    pub const fn advance_default(&mut self) -> u8 {
        if self.is_at_end() { 0 } else { unsafe { self.advance_unchecked() } }
    }

    /// # Safety
    ///
    /// `self.is_at_end()` must be false.
    #[inline]
    pub const unsafe fn peek_next_unchecked(&self) -> u8 {
        unsafe {
            assert_unchecked(self.index + 1 < self.source.len());
            *self.source.as_bytes().as_ptr().add(self.index + 1)
        }
    }

    #[inline]
    pub const fn peek_next(&self) -> Option<u8> {
        if self.index + 1 >= self.source.len() {
            None
        } else {
            Some(unsafe { self.peek_next_unchecked() })
        }
    }

    #[inline]
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

    /// # Safety
    ///
    /// - the entirety of the range `self.source.as_bytes[self.start..self.index]` must
    /// be in bounds.
    /// 
    /// NOTE: `self.index` may equal `self.source.len()` and does not pose a problem.
    #[inline]
    pub const unsafe fn slice_here(&self) -> &'source [u8] {
        unsafe {
            let ptr = self.source.as_bytes().as_ptr().add(self.start);
            let len = self.index - self.start;
            slice::from_raw_parts(ptr, len)
        }
    }
}
