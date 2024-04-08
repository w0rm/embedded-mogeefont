use embedded_graphics::{
    geometry::{Point, Size},
    image::{ImageDrawableExt, ImageRaw, SubImage},
    mono_font::mapping::{GlyphMapping, StrGlyphMapping},
    pixelcolor::BinaryColor,
    primitives::Rectangle,
};

use crate::ligature_substitution::StrLigatureSubstitution;

pub struct Font<'a> {
    pub(crate) image: ImageRaw<'a, BinaryColor>,
    pub(crate) glyph_mapping: StrGlyphMapping<'a>,
    pub(crate) glyph_data: &'a [u8],
    pub(crate) ligature_substitution: StrLigatureSubstitution<'a>,
    pub(crate) character_height: u32,
    pub(crate) baseline: u32,
    pub(crate) character_spacing: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GlyphIndex(usize);

impl<'a> Font<'a> {
    pub fn char_offsets<'t>(&'a self, text: &'t str) -> impl Iterator<Item = GlyphIndex> + 't
    where
        'a: 't,
    {
        // Iterate over the characters in the text and return the glyph index for each character
        // the ligatures are handled by the ligature substitution.
        let mut chars = text.as_bytes();
        core::iter::from_fn(move || {
            if chars.is_empty() {
                return None;
            }
            if let Some((liga_index, offset)) = self.ligature_substitution.substitute(chars) {
                chars = offset;
                return Some(GlyphIndex(liga_index));
            } else {
                let (glyph, rest) = chars.split_first()?;
                chars = rest;
                return Some(self.glyph_index(*glyph as char));
            }
        })
    }

    /// Returns the glyph index for a character.
    fn glyph_index(&self, glyph: char) -> GlyphIndex {
        GlyphIndex(self.glyph_mapping.index(glyph))
    }

    /// Returns the area of a glyph in the font image.
    fn glyph_area(&self, index: GlyphIndex) -> Option<Rectangle> {
        let start = index.0 * 4;
        let end = start + 4;
        if let [x, y, width, height] = self.glyph_data[start..end] {
            Some(Rectangle::new(
                Point::new(x as i32, y as i32),
                Size::new(width as u32, height as u32),
            ))
        } else {
            None
        }
    }

    /// Returns the area of a glyph in the font image.
    pub fn glyph_width(&self, index: GlyphIndex) -> i32 {
        self.glyph_data[index.0 * 4 + 2].into()
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
            Some(Rectangle::new(Point::zero(), Size::new(1, 11)))
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
        let text = "ffifijjjssyj";
        let ligatures_offset = 160;
        let mut chars = MOGEEFONT.char_offsets(text);
        assert_eq!(chars.next(), Some(GlyphIndex(ligatures_offset + 0))); // ffi
        assert_eq!(chars.next(), Some(GlyphIndex(ligatures_offset + 2))); // fi
        assert_eq!(chars.next(), Some(GlyphIndex(ligatures_offset + 5))); // jj
        assert_eq!(chars.next(), Some(GlyphIndex(73))); // j
        assert_eq!(chars.next(), Some(GlyphIndex(ligatures_offset + 6))); // ss
        assert_eq!(chars.next(), Some(GlyphIndex(ligatures_offset + 7))); // yj
        assert_eq!(chars.next(), None);
    }
}
