use crate::charset::GlyphIndex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kerning<'a> {
    pairs: &'a [(u8, u8, i32)],
    overrides: &'a [(usize, usize, i32)],
}

impl<'a> Kerning<'a> {
    pub const fn new(pairs: &'a [(u8, u8, i32)], overrides: &'a [(usize, usize, i32)]) -> Self {
        Self { pairs, overrides }
    }

    pub fn kerning_override(&self, left: GlyphIndex, right: GlyphIndex) -> Option<i32> {
        self.overrides
            .binary_search_by_key(&(left.0, right.0), |(l, r, _)| (*l, *r))
            .map(|i| self.overrides[i].2)
            .ok()
    }

    pub fn kerning(&self, left_class: u8, right_class: u8) -> Option<i32> {
        self.pairs
            .binary_search_by_key(&(left_class, right_class), |(l, r, _)| (*l, *r))
            .map(|i| self.pairs[i].2)
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kerning_override() {
        let kerning = Kerning::new(&[], &[(1, 2, 1), (3, 4, -1)]);
        assert_eq!(
            kerning.kerning_override(GlyphIndex(1), GlyphIndex(2)),
            Some(1)
        );
        assert_eq!(
            kerning.kerning_override(GlyphIndex(3), GlyphIndex(4)),
            Some(-1)
        );
        assert_eq!(kerning.kerning_override(GlyphIndex(5), GlyphIndex(6)), None);
    }

    #[test]
    fn test_kerning() {
        let kerning = Kerning::new(&[(1, 2, 0), (3, 4, -1)], &[]);
        assert_eq!(kerning.kerning(1, 2), Some(0));
        assert_eq!(kerning.kerning(3, 4), Some(-1));
        assert_eq!(kerning.kerning(5, 6), None);
    }
}
