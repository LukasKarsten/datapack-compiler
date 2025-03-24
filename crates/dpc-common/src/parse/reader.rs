use std::ops::Range;

#[derive(Clone)]
pub struct Reader<'a> {
    src: &'a str,
    pos: usize,
    cur: Option<char>,
}

impl<'a> Reader<'a> {
    pub fn new(src: &'a str) -> Self {
        Self::with_pos(src, 0)
    }

    pub fn with_pos(src: &'a str, pos: usize) -> Self {
        Self {
            src,
            pos,
            cur: src[pos..].chars().next(),
        }
    }

    pub fn with_range(src: &'a str, range: Range<usize>) -> Self {
        Self::with_pos(&src[..range.end], range.start)
    }

    pub fn parse_with_span<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> (Range<usize>, R) {
        let start = self.pos;
        let result = f(self);
        (start..self.pos, result)
    }

    pub fn try_parse_with_span<T, E>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<(Range<usize>, T), E> {
        let (span, result) = self.parse_with_span(f);
        Ok((span, result?))
    }

    pub fn remaining_src(&self) -> &'a str {
        &self.src[self.pos..]
    }

    pub fn get_src(&self) -> &'a str {
        self.src
    }

    pub fn get_pos(&self) -> usize {
        self.pos
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
        self.cur = self.src[pos..].chars().next();
    }

    pub fn get_next_pos(&self) -> usize {
        self.pos + self.cur.map_or(0, char::len_utf8)
    }

    pub fn has_more(&self) -> bool {
        self.cur.is_some()
    }

    pub fn peek(&self) -> Option<char> {
        self.cur
    }

    pub fn peek2(&self) -> Option<char> {
        let mut clone = self.clone();
        clone.advance();
        clone.peek()
    }

    pub fn advance(&mut self) {
        if let Some(chr) = self.cur {
            self.pos += chr.len_utf8();
            self.cur = unsafe { self.src.get_unchecked(self.pos..).chars().next() };
        }
    }

    pub fn skip(&mut self, string: &str) -> bool {
        if self.remaining_src().starts_with(string) {
            self.set_pos(self.pos + string.len());
            true
        } else {
            false
        }
    }

    pub fn skip_whitespace(&mut self) {
        self.read_span_while(|chr| chr.is_whitespace());
    }

    pub fn read_range_until(&mut self, mut f: impl FnMut(char) -> bool) -> Range<usize> {
        let start = self.pos;
        while !self.peek().is_none_or(&mut f) {
            self.advance();
        }
        start..self.pos
    }

    pub fn read_until(&mut self, f: impl FnMut(char) -> bool) -> &'a str {
        &self.src[self.read_range_until(f)]
    }

    pub fn read_span_while(&mut self, mut f: impl FnMut(char) -> bool) -> Range<usize> {
        self.read_range_until(|chr| !f(chr))
    }

    pub fn read_while(&mut self, f: impl FnMut(char) -> bool) -> &'a str {
        &self.src[self.read_span_while(f)]
    }

    pub fn read_literal(&mut self) -> &'a str {
        self.read_until(|chr| chr.is_whitespace())
    }
}
