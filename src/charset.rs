use crate::{kerning::Kerning, ligatures::Ligatures, side_bearings::SideBearings};
use embedded_graphics::{
    geometry::{Point, Size},
    image::ImageRaw,
    mono_font::mapping::{GlyphMapping, StrGlyphMapping},
    pixelcolor::BinaryColor,
    primitives::Rectangle,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Charset {
    pub(crate) image: ImageRaw<'static, BinaryColor>,
    pub(crate) glyph_mapping: StrGlyphMapping<'static>,
    pub(crate) glyph_data: &'static [u8],
    pub(crate) ligatures: Ligatures<'static>,
    pub(crate) side_bearings: SideBearings<'static>,
    pub(crate) kerning: Kerning<'static>,
    pub(crate) line_height: u32,
    pub(crate) baseline: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlyphIndex(pub(crate) usize);

impl Charset {
    /// Returns an iterator over the glyph indices for the characters in a text.
    /// Performs ligature substitution.
    pub(crate) fn glyph_indices<'t>(
        &'static self,
        text: &'t str,
    ) -> impl Iterator<Item = GlyphIndex> + 't {
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

    /// Returns the area of a glyph in the font image.
    pub(crate) fn glyph_area(&self, glyph: GlyphIndex) -> Rectangle {
        let start = glyph.0 * 5;
        let end = start + 3;
        if let [x, y, dimensions] = self.glyph_data[start..end] {
            let width = (dimensions & 0x0F) as u32; // Lower 4 bits
            let height = (dimensions >> 4) as u32; // Upper 4 bits
            Rectangle::new(Point::new(x as i32, y as i32), Size::new(width, height))
        } else {
            Rectangle::zero()
        }
    }

    /// Returns the width of a glyph in the font image.
    pub(crate) fn glyph_width(&self, index: GlyphIndex) -> i32 {
        (self.glyph_data[index.0 * 5 + 2] & 0x0F) as i32
    }

    /// Returns the spacing between two glyphs, or between the start of the text and a glyph.
    /// Takes into account the side bearings and kerning.
    pub(crate) fn spacing(&self, prev_glyph: Option<GlyphIndex>, next_glyph: GlyphIndex) -> i32 {
        match prev_glyph {
            Some(prev) => {
                let right_bearing = self.side_bearings.right(prev);
                let left_bearing = self.side_bearings.left(next_glyph);
                let kerning = self.kerning(prev, next_glyph).unwrap_or_default();
                right_bearing + kerning + left_bearing
            }
            None => self.side_bearings.left(next_glyph),
        }
    }

    /// Returns the glyph index for a character.
    fn glyph_index(&self, char: char) -> GlyphIndex {
        GlyphIndex(self.glyph_mapping.index(char))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::ASCII;
    use embedded_graphics::{image::ImageDrawable, mock_display::MockDisplay};

    #[test]
    fn test_glyph() {
        let area = ASCII.glyph_area(ASCII.glyph_index('a'));
        let mut display = MockDisplay::new();
        ASCII.image.draw_sub_image(&mut display, &area).unwrap();
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
            ASCII.glyph_area(ASCII.glyph_index('a')),
            Rectangle::new(Point::new(58, 24), Size::new(3, 11))
        );
        assert_eq!(
            ASCII.glyph_area(ASCII.glyph_index('!')),
            Rectangle::new(Point::new(4, 0), Size::new(1, 11))
        );
        assert_eq!(
            ASCII.glyph_area(ASCII.glyph_index('熊')),
            ASCII.glyph_area(ASCII.glyph_index('?'))
        );
    }

    #[test]
    fn test_ligature_substitution_in_text() {
        let text = "虫ffifijjjssyj";
        let ligatures_offset = ASCII.ligatures.offset;
        let mut glyphs = ASCII.glyph_indices(text);
        assert_eq!(glyphs.next(), Some(ASCII.glyph_index('虫')));
        assert_eq!(glyphs.next(), Some(GlyphIndex(ligatures_offset + 0))); // ffi
        assert_eq!(glyphs.next(), Some(GlyphIndex(ligatures_offset + 2))); // fi
        assert_eq!(glyphs.next(), Some(GlyphIndex(ligatures_offset + 5))); // jj
        assert_eq!(glyphs.next(), Some(ASCII.glyph_index('j')));
        assert_eq!(glyphs.next(), Some(GlyphIndex(ligatures_offset + 6))); // ss
        assert_eq!(glyphs.next(), Some(GlyphIndex(ligatures_offset + 7))); // yj
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_letter_spacing() {
        assert_eq!(
            ASCII.spacing(Some(ASCII.glyph_index('o')), ASCII.glyph_index(',')),
            0
        );
        assert_eq!(
            ASCII.kerning(ASCII.glyph_index('f'), ASCII.glyph_index('o')),
            Some(-1)
        );
    }

    #[test]
    fn test_kerning() {
        assert_eq!(
            ASCII.kerning(ASCII.glyph_index('s'), ASCII.glyph_index('`')),
            Some(-1)
        );
        assert_eq!(
            ASCII.kerning(ASCII.glyph_index('f'), ASCII.glyph_index('o')),
            Some(-1)
        );
        assert_eq!(
            ASCII.kerning(ASCII.glyph_index('T'), ASCII.glyph_index('/')),
            Some(-2)
        );
        assert_eq!(
            ASCII.kerning(ASCII.glyph_index('/'), ASCII.glyph_index('/')),
            Some(-2)
        );
        assert_eq!(
            ASCII.kerning(ASCII.glyph_index('o'), ASCII.glyph_index(',')),
            Some(-1)
        );
        assert_eq!(
            ASCII.kerning(ASCII.glyph_index('u'), ASCII.glyph_index('s')),
            None
        );
    }

    #[test]
    fn test_kerning_classes() {
        assert_eq!(
            ASCII.right_kerning_class(ASCII.glyph_index('Y')),
            ASCII.right_kerning_class(ASCII.glyph_index('`'))
        );
        assert_eq!(
            ASCII.right_kerning_class(ASCII.glyph_index('"')),
            ASCII.right_kerning_class(ASCII.glyph_index('\''))
        );
        assert_eq!(ASCII.left_kerning_class(ASCII.glyph_index('o')), 16);
        assert_eq!(ASCII.right_kerning_class(ASCII.glyph_index(',')), 14);
    }

    #[test]
    fn test_side_bearings() {
        assert_eq!(ASCII.side_bearings.left(ASCII.glyph_index(',')), 0);
        assert_eq!(ASCII.side_bearings.right(ASCII.glyph_index('o')), 1);
    }
}
