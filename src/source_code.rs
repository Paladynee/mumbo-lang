#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceCode<'source> {
    code: &'source str,
}

impl<'source> SourceCode<'source> {
    #[inline]
    pub const fn new(code: &'source str) -> Self {
        SourceCode { code }
    }

    #[inline(always)]
    pub const fn as_str(&self) -> &'source str {
        self.code
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.code.len()
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub const fn as_bytes(&self) -> &'source [u8] {
        self.code.as_bytes()
    }
}
