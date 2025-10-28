use crate::lexer::Lexer;
use crate::types::Token;
use core::slice;

#[inline(always)]
pub const fn const_index(s: &[u8], index: usize) -> Option<u8> {
    if index >= s.len() {
        None
    } else {
        Some(unsafe { *s.as_ptr().add(index) })
    }
}

#[inline(always)]
pub const fn const_subslice(s: &[u8], start: usize, end: usize) -> Option<&[u8]> {
    if start > end || end > s.len() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts(s.as_ptr().add(start), end - start) })
    }
}

/// trie implementation
///
/// # Safety
///
/// s.len() must be at least 1.
pub const unsafe fn check_identifier_actual_token<'src>(lexer: &mut Lexer<'src>, s: &'src [u8]) -> Token {
    let r = match unsafe { *s.as_ptr() } {
        b'l' => {
            // let
            identifier_check_rest(s, 1, b"et", Token::KwLet)
        }
        b'f' => {
            // fn
            identifier_check_rest(s, 1, b"n", Token::KwFn)
        }
        b'r' => {
            // return, runtime
            let Some(next) = const_index(s, 1) else {
                return Token::LitIdentifier;
            };
            match next {
                b'e' => identifier_check_rest(s, 2, b"turn", Token::KwReturn),
                b'u' => identifier_check_rest(s, 2, b"ntime", Token::KwRuntime),
                _ => Token::LitIdentifier,
            }
        }
        b'e' => {
            // extern, enum
            let Some(next) = const_index(s, 1) else {
                return Token::LitIdentifier;
            };
            match next {
                b'x' => identifier_check_rest(s, 2, b"tern", Token::KwExtern),
                b'n' => identifier_check_rest(s, 2, b"um", Token::KwAdtEnum),
                _ => Token::LitIdentifier,
            }
        }
        b'c' => {
            // const, compiletime, cast
            let Some(next) = const_index(s, 1) else {
                return Token::LitIdentifier;
            };
            match next {
                b'o' => {
                    let Some(next) = const_index(s, 2) else {
                        return Token::LitIdentifier;
                    };
                    match next {
                        b'n' => identifier_check_rest(s, 3, b"st", Token::KwConst),
                        b'm' => identifier_check_rest(s, 3, b"piletime", Token::KwCompiletime),
                        _ => Token::LitIdentifier,
                    }
                }
                b'a' => identifier_check_rest(s, 2, b"st", Token::KwCast),
                _ => Token::LitIdentifier,
            }
        }
        b'm' => {
            // mut
            identifier_check_rest(s, 1, b"ut", Token::KwMut)
        }
        b'a' => {
            // anymut
            identifier_check_rest(s, 1, b"nymut", Token::KwAnymut)
        }
        b's' => {
            // static, struct
            let Some(next) = const_index(s, 1) else {
                return Token::LitIdentifier;
            };
            match next {
                b't' => {
                    let Some(next) = const_index(s, 2) else {
                        return Token::LitIdentifier;
                    };

                    match next {
                        b'a' => identifier_check_rest(s, 3, b"tic", Token::KwStatic),
                        b'r' => identifier_check_rest(s, 3, b"uct", Token::KwAdtStruct),
                        _ => Token::LitIdentifier,
                    }
                }
                _ => Token::LitIdentifier,
            }
        }
        b't' => {
            // type
            identifier_check_rest(s, 1, b"ype", Token::KwType)
        }
        b'u' => {
            // union, uninit
            let Some(next) = const_index(s, 1) else {
                return Token::LitIdentifier;
            };

            match next {
                b'n' => {
                    let Some(next) = const_index(s, 2) else {
                        return Token::LitIdentifier;
                    };

                    match next {
                        b'i' => {
                            let Some(next) = const_index(s, 3) else {
                                return Token::LitIdentifier;
                            };

                            match next {
                                b'n' => identifier_check_rest(s, 4, b"it", Token::LitUninit),
                                b'o' => identifier_check_rest(s, 4, b"n", Token::KwAdtUnion),
                                _ => Token::LitIdentifier,
                            }
                        }
                        _ => Token::LitIdentifier,
                    }
                }
                _ => Token::LitIdentifier,
            }
        }
        _ => Token::LitIdentifier,
    };

    if r.is_identifier_extractable() {
        lexer.literal = Some(s);
    }

    r
}

#[inline]
pub const fn const_slice_eq(s1: &[u8], s2: &[u8]) -> bool {
    if s1.len() != s2.len() {
        return false;
    }

    let mut index = 0;

    while index < s1.len() {
        if s1[index] != s2[index] {
            return false;
        }
        index += 1;
    }

    true
}

#[inline]
pub const fn identifier_check_rest(s: &[u8], start: usize, rest: &'static [u8], token: Token) -> Token {
    let Some(ss) = const_subslice(s, start, s.len()) else {
        return Token::LitIdentifier;
    };

    if const_slice_eq(ss, rest) { token } else { Token::LitIdentifier }
}

#[inline]
pub const fn is_valid_identifier_tail(byte: u8) -> bool {
    matches!(
        byte,
        b'a'..=b'z' | b'A' ..=b'Z' | b'_' | b'0' ..=b'9'
    )
}

#[inline]
pub const fn is_valid_identifier_head(byte: u8) -> bool {
    matches!(
        byte,
        b'a'..=b'z' | b'A' ..=b'Z' | b'_'
    )
}
