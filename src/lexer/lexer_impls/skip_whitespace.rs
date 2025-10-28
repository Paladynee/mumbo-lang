use crate::lexer::Lexer;

pub const fn skip_whitespace_impl(lexer: &mut Lexer<'_>) {
    while !lexer.is_at_end() {
        // SAFETY: we are guaranteed to not be at the end here

        let next = unsafe { lexer.peek_unchecked() };

        match next {
            b' ' | b'\r' | b'\t' | b'\n' => unsafe {
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
