pub struct StrLigatureSubstitution<'a> {
    data: &'a str,
    offset: usize,
}

impl<'a> StrLigatureSubstitution<'a> {
    /// Creates a new glyph mapping.
    pub const fn new(data: &'a str, offset: usize) -> Self {
        Self { data, offset }
    }

    /// Return the index of the ligature glyph
    /// and the number of chars to skip
    /// if the string starts with a ligature.
    pub fn substitute(&self, str: &str) -> Option<(usize, usize)> {
        let mut offset = self.offset;
        for liga in self.data.split('\0') {
            if liga.is_empty() {
                continue;
            }
            if str.starts_with(liga) {
                return Some((offset, liga.len()));
            }
            offset += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute() {
        let offset = 31;
        let mapping = StrLigatureSubstitution::new("\0\u{66}\u{66}\u{69}\0\u{66}\u{66}\0\u{66}\u{69}\0\u{66}\u{6a}\0\u{67}\u{6a}\0\u{6a}\u{6a}\0\u{73}\u{73}\0\u{79}\u{6a}", offset);
        assert_eq!(mapping.substitute("f"), None);
        assert_eq!(mapping.substitute("ffi"), Some((offset + 0, 3)));
        assert_eq!(mapping.substitute("ff"), Some((offset + 1, 2)));
        assert_eq!(mapping.substitute("fi"), Some((offset + 2, 2)));
        assert_eq!(mapping.substitute("yj"), Some((offset + 7, 2)));
    }
}
