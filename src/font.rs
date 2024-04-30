use crate::{kerning::Kerning, ligatures::Ligatures, side_bearings::SideBearings};
use embedded_graphics::{
    geometry::{Point, Size},
    image::{ImageDrawableExt, ImageRaw, SubImage},
    mono_font::mapping::{GlyphMapping, StrGlyphMapping},
    pixelcolor::BinaryColor,
    primitives::Rectangle,
};

pub struct Font<'a> {
    pub(crate) image: ImageRaw<'a, BinaryColor>,
    pub(crate) glyph_mapping: StrGlyphMapping<'a>,
    pub(crate) glyph_data: &'a [u8],
    pub(crate) ligatures: Ligatures<'a>,
    pub(crate) side_bearings: SideBearings<'a>,
    pub(crate) kerning: Kerning<'a>,
    pub(crate) em_height: u32,
    pub(crate) baseline: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GlyphIndex(pub(crate) usize);

impl<'a> Font<'a> {
    pub fn char_offsets<'t>(&'a self, text: &'t str) -> impl Iterator<Item = GlyphIndex> + 't
    where
        'a: 't,
    {
        // Iterate over the characters in the text and return the glyph index for each character
        // the ligatures are handled by the ligature substitution.
        let mut byte_offset = 0;
        let mut chars = text.chars();
        core::iter::from_fn(move || {
            if let Some((liga_index, mut liga_size)) =
                self.ligatures.substitute(&text[byte_offset..])
            {
                // Advance by the number of characters in the ligature
                while liga_size > 0 {
                    if let Some(glyph) = chars.next() {
                        byte_offset += glyph.len_utf8();
                    } else {
                        return None;
                    }
                    liga_size -= 1;
                }
                return Some(GlyphIndex(liga_index));
            } else if let Some(glyph) = chars.next() {
                byte_offset += glyph.len_utf8();
                return Some(self.glyph_index(glyph));
            } else {
                return None;
            }
        })
    }

    /// Returns the glyph index for a character.
    fn glyph_index(&self, glyph: char) -> GlyphIndex {
        GlyphIndex(self.glyph_mapping.index(glyph))
    }

    /// Returns the area of a glyph in the font image.
    fn glyph_area(&self, index: GlyphIndex) -> Option<Rectangle> {
        let start = index.0 * 5;
        let end = start + 3;
        if let [x, y, dimensions] = self.glyph_data[start..end] {
            let width = (dimensions & 0x0F) as u32; // Lower 4 bits
            let height = (dimensions >> 4) as u32; // Upper 4 bits
            Some(Rectangle::new(
                Point::new(x as i32, y as i32),
                Size::new(width, height),
            ))
        } else {
            None
        }
    }

    pub fn letter_spacing(&self, prev_char: Option<GlyphIndex>, next_char: GlyphIndex) -> i32 {
        match prev_char {
            Some(prev) => {
                let right_bearing = self.side_bearings.right(prev);
                let left_bearing = self.side_bearings.left(next_char);
                let kerning = self.kerning(prev, next_char).unwrap_or_default();
                right_bearing + kerning + left_bearing
            }
            None => self.side_bearings.left(next_char),
        }
    }

    /// Returns the kerning between two glyphs.
    fn kerning(&self, left: GlyphIndex, right: GlyphIndex) -> Option<i32> {
        self.kerning.kerning_override(left, right).or_else(|| {
            self.kerning.kerning(
                self.left_kerning_class(left),
                self.right_kerning_class(right),
            )
        })
    }

    /// Returns the left kerning class for a glyph.
    /// The left kerning class is stored in the upper 4 bits of the byte.
    fn left_kerning_class(&self, index: GlyphIndex) -> u8 {
        self.glyph_data[index.0 * 5 + 3]
    }

    /// Returns the right kerning class for a glyph.
    /// The right kerning class is stored in the lower 4 bits of the byte.
    fn right_kerning_class(&self, index: GlyphIndex) -> u8 {
        self.glyph_data[index.0 * 5 + 4]
    }

    /// Returns the width of a glyph in the font image.
    pub fn glyph_width(&self, index: GlyphIndex) -> i32 {
        (self.glyph_data[index.0 * 5 + 2] & 0x0F) as i32
    }

    /// Returns a subimage for a glyph.
    pub fn glyph(&self, index: GlyphIndex) -> SubImage<'a, ImageRaw<BinaryColor>> {
        self.glyph_area(index)
            .map(|area| self.image.sub_image(&area))
            .unwrap_or_else(|| self.image.sub_image(&Rectangle::zero()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::MOGEEFONT;
    use embedded_graphics::{image::ImageDrawable, mock_display::MockDisplay};

    #[test]
    fn test_glyph() {
        let sub_image = MOGEEFONT.glyph(MOGEEFONT.glyph_index('a'));
        let mut display = MockDisplay::new();
        sub_image.draw(&mut display).unwrap();
        display.assert_pattern(&[
            "...", //
            "...", //
            "...", //
            "##.", //
            "..#", //
            "###", //
            "#.#", //
            ".##", //
            "...", //
            "...", //
            "...", //
        ]);
    }

    #[test]
    fn test_glyph_area() {
        assert_eq!(
            MOGEEFONT.glyph_area(MOGEEFONT.glyph_index('a')),
            Some(Rectangle::new(Point::new(55, 24), Size::new(3, 11)))
        );
        assert_eq!(
            MOGEEFONT.glyph_area(MOGEEFONT.glyph_index('!')),
            Some(Rectangle::new(Point::new(4, 0), Size::new(1, 11)))
        );
        assert_eq!(
            MOGEEFONT.glyph_area(MOGEEFONT.glyph_index('ё')),
            Some(Rectangle::new(Point::new(123, 60), Size::new(3, 11)))
        );
        assert_eq!(
            MOGEEFONT.glyph_area(MOGEEFONT.glyph_index('熊')),
            MOGEEFONT.glyph_area(MOGEEFONT.glyph_index('?'))
        );
    }

    #[test]
    fn test_ligature_substitution_in_text() {
        let text = "虫ffifijjjssyj";
        let ligatures_offset = 160;
        let mut chars = MOGEEFONT.char_offsets(text);
        assert_eq!(chars.next(), Some(MOGEEFONT.glyph_index('虫')));
        assert_eq!(chars.next(), Some(GlyphIndex(ligatures_offset + 0))); // ffi
        assert_eq!(chars.next(), Some(GlyphIndex(ligatures_offset + 2))); // fi
        assert_eq!(chars.next(), Some(GlyphIndex(ligatures_offset + 5))); // jj
        assert_eq!(chars.next(), Some(MOGEEFONT.glyph_index('j')));
        assert_eq!(chars.next(), Some(GlyphIndex(ligatures_offset + 6))); // ss
        assert_eq!(chars.next(), Some(GlyphIndex(ligatures_offset + 7))); // yj
        assert_eq!(chars.next(), None);
    }

    #[test]
    fn test_letter_spacing() {
        assert_eq!(
            MOGEEFONT.letter_spacing(Some(MOGEEFONT.glyph_index('o')), MOGEEFONT.glyph_index(',')),
            0
        );
        assert_eq!(
            MOGEEFONT.kerning(MOGEEFONT.glyph_index('f'), MOGEEFONT.glyph_index('o')),
            Some(-1)
        );
    }

    #[test]
    fn test_kerning() {
        assert_eq!(
            MOGEEFONT.kerning(MOGEEFONT.glyph_index('f'), MOGEEFONT.glyph_index('o')),
            Some(-1)
        );
        assert_eq!(
            MOGEEFONT.kerning(MOGEEFONT.glyph_index('T'), MOGEEFONT.glyph_index('/')),
            Some(-2)
        );
        assert_eq!(
            MOGEEFONT.kerning(MOGEEFONT.glyph_index('/'), MOGEEFONT.glyph_index('/')),
            Some(-2)
        );
        assert_eq!(
            MOGEEFONT.kerning(MOGEEFONT.glyph_index('o'), MOGEEFONT.glyph_index(',')),
            Some(-1)
        );
        assert_eq!(
            MOGEEFONT.kerning(MOGEEFONT.glyph_index('u'), MOGEEFONT.glyph_index('s')),
            None
        );
    }

    #[test]
    fn test_kerning_classes() {
        assert_eq!(MOGEEFONT.left_kerning_class(MOGEEFONT.glyph_index('o')), 16);
        assert_eq!(
            MOGEEFONT.right_kerning_class(MOGEEFONT.glyph_index(',')),
            14
        );
    }

    #[test]
    fn test_side_bearings() {
        assert_eq!(MOGEEFONT.side_bearings.left(MOGEEFONT.glyph_index(',')), 0);
        assert_eq!(MOGEEFONT.side_bearings.right(MOGEEFONT.glyph_index('o')), 1);
    }
}
