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
    pub(crate) ligature_code_points: &'a str,
    pub(crate) character_height: u32,
    pub(crate) baseline: u32,
    pub(crate) character_spacing: i32,
}

impl<'a> Font<'a> {
    /// Returns the area of a glyph in the font image.
    fn glyph_area(&self, glyph: char) -> Option<Rectangle> {
        let index = self.glyph_mapping.index(glyph);
        let start = index * 4;
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
    pub fn glyph_width(&self, glyph: char) -> i32 {
        let index = self.glyph_mapping.index(glyph);
        self.glyph_data[index * 4 + 2].into()
    }

    /// Returns a subimage for a glyph.
    pub fn glyph(&self, c: char) -> SubImage<'a, ImageRaw<BinaryColor>> {
        self.glyph_area(c)
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
        let sub_image = MOGEEFONT.glyph('a');
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
            MOGEEFONT.glyph_area('a'),
            Some(Rectangle::new(Point::new(55, 24), Size::new(3, 11)))
        );
        assert_eq!(
            MOGEEFONT.glyph_area('!'),
            Some(Rectangle::new(Point::zero(), Size::new(1, 11)))
        );
        assert_eq!(
            MOGEEFONT.glyph_area('ё'),
            Some(Rectangle::new(Point::new(123, 60), Size::new(3, 11)))
        );
        assert_eq!(MOGEEFONT.glyph_area('熊'), MOGEEFONT.glyph_area('?'));
    }
}
