use crate::lexer::{Lexer, lexer_impls};

pub const fn skip_whitespace_impl(lexer: &mut Lexer<'_>) {
    while !lexer.is_at_end() {
        // SAFETY: we are guaranteed to not be at the end here

        let next = unsafe { lexer.peek_unchecked() };

        match next {
            c if lexer_impls::skip_whitespace::is_whitespace(c) => unsafe {
                lexer.advance_unchecked();
            },

            b'/' => {
                if let Some(byte) = lexer.peek_next()
                    && byte == b'/'
                {
                    unsafe {
                        lexer.advance_unchecked();
                        lexer.advance_unchecked();
                    };

                    // we could be at end here

                    while !lexer.is_at_end() {
                        // SAFETY: we are guaranteed to not be at the end here

                        let byte = unsafe { lexer.peek_unchecked() };
                        if byte != b'\n' {
                            unsafe { lexer.advance_unchecked() };
                        } else {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }

            _ => break,
        };
    }
}

#[inline]
pub const fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\r' | b'\t' | b'\n')
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, source_code::SourceCode};

    #[test]
    fn skips_whitespace_correctly() {
        let source = "
            hi
            // residual
        ";

        let mut lexer = Lexer::new(SourceCode::new(source));

        lexer.skip_whitespace();
        assert!(!lexer.is_at_end());
        assert!(lexer.matches_bytes(b"hi"));
        assert!(!lexer.is_at_end());
        assert_eq!(lexer.peek(), Some(b'\n'));

        lexer.skip_whitespace();
        assert!(lexer.is_at_end());
        assert_eq!(lexer.peek(), None);
    }
}
