pub struct StrLigatureSubstitution<'a> {
    data: &'a str,
    offset: usize,
}

impl<'a> StrLigatureSubstitution<'a> {
    /// Creates a new glyph mapping.
    pub const fn new(data: &'a str, offset: usize) -> Self {
        Self { data, offset }
    }

    /// Return an iterator over ligatures
    /// The ligatures are separated by a null byte
    fn ligatures(&self) -> impl Iterator<Item = &[u8]> {
        let chars = self.data.as_bytes();
        let mut start = 0;
        let mut end = 0;
        core::iter::from_fn(move || {
            while start < chars.len() {
                while end < chars.len() && chars[end] != 0 {
                    end += 1;
                }
                let ligature = &chars[start..end];
                start = end + 1;
                end = start;
                if !ligature.is_empty() {
                    return Some(ligature);
                }
            }
            None
        })
    }

    /// Return the index of the ligature glyph and the remaining string
    /// if the string starts with a ligature.
    pub fn substitute(&self, str: &'a [u8]) -> Option<(usize, &'a [u8])> {
        let mut offset = self.offset;
        for chunk in self.ligatures() {
            if str.starts_with(chunk) {
                return Some((offset, &str[chunk.len()..]));
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
        assert_eq!(mapping.substitute(&['f' as u8]), None);
        assert_eq!(
            mapping.substitute("ffi".as_bytes()),
            Some((offset + 0, "".as_bytes()))
        );
        assert_eq!(
            mapping.substitute("ff".as_bytes()),
            Some((offset + 1, "".as_bytes()))
        );
        assert_eq!(
            mapping.substitute("fi".as_bytes()),
            Some((offset + 2, "".as_bytes()))
        );
        assert_eq!(
            mapping.substitute("yj".as_bytes()),
            Some((offset + 7, "".as_bytes()))
        );
    }
}
