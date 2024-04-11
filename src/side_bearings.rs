use crate::font::GlyphIndex;
use az::SaturatingAs;

pub struct SideBearings<'a> {
    bearings: &'a [(u32, i32, i32)],
    default_bearings: (i32, i32),
}

impl<'a> SideBearings<'a> {
    pub const fn new(bearings: &'a [(u32, i32, i32)], default_bearings: (i32, i32)) -> Self {
        Self {
            bearings,
            default_bearings,
        }
    }

    pub fn left(&self, index: GlyphIndex) -> i32 {
        self.bearings
            .binary_search_by_key(&(index.0.saturating_as()), |data| data.0)
            .map(|idx| self.bearings[idx].1)
            .unwrap_or(self.default_bearings.0)
    }

    pub fn right(&self, index: GlyphIndex) -> i32 {
        self.bearings
            .binary_search_by_key(&(index.0.saturating_as()), |data| data.0)
            .map(|idx| self.bearings[idx].2)
            .unwrap_or(self.default_bearings.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_bearings() {
        let bearings = SideBearings::new(&[(0, 1, 2), (1, 3, 4)], (5, 6));

        assert_eq!(bearings.left(GlyphIndex(0)), 1);
        assert_eq!(bearings.right(GlyphIndex(0)), 2);

        assert_eq!(bearings.left(GlyphIndex(1)), 3);
        assert_eq!(bearings.right(GlyphIndex(1)), 4);

        assert_eq!(bearings.left(GlyphIndex(2)), 5);
        assert_eq!(bearings.right(GlyphIndex(2)), 6);
    }
}
