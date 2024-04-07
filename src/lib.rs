mod mogeefont;

use embedded_graphics::mono_font::mapping::GlyphMapping;
use mogeefont::*;

pub struct MogeeFont<C> {
    /// Text color.
    pub text_color: Option<C>,
}

impl<C> MogeeFont<C> {
    pub fn new(text_color: C) -> Self {
        Self {
            text_color: Some(text_color),
        }
    }

    fn get_glyph_coords(&self, glyph: char) -> Option<(u32, u32, u32, u32)> {
        let index = MOGEEFONT_GLYPH_MAPPING.index(glyph);
        let start = index * 4;
        let end = start + 4;
        let coords = &MOGEEFONT_GLYPH_DATA[start..end];
        let x = coords[0] as u32;
        let y = coords[1] as u32;
        let width = coords[2] as u32;
        let height = coords[3] as u32;
        Some((x, y, width, height))
    }
}

// test
#[cfg(test)]
mod tests {
    use embedded_graphics::mono_font::mapping::GlyphMapping;

    use super::*;

    #[test]
    fn test_glyph_mapping() {
        let index = MOGEEFONT_GLYPH_MAPPING.index('a');
        assert_eq!(index, 63);
    }

    #[test]
    fn test_glyph_coords() {
        let font = MogeeFont::new(());
        assert_eq!(font.get_glyph_coords('a'), Some((55, 24, 3, 11)));
        assert_eq!(font.get_glyph_coords('!'), Some((0, 0, 1, 11)));
        assert_eq!(font.get_glyph_coords('ё'), Some((123, 60, 3, 11)));
        assert_eq!(font.get_glyph_coords('熊'), font.get_glyph_coords('?'));
    }
}
