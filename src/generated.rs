use crate::font::Font;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::mono_font::mapping::StrGlyphMapping;

/// ![mogeefont](data:image/png;base64,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)
pub const MOGEEFONT: Font<'_> = Font {
    image: ImageRaw::new(include_bytes!("mogeefont.raw"), 128),
    glyph_mapping: StrGlyphMapping::new(
        "\0\u{21}\u{5f}\0\u{61}\u{7e}\u{d7}\u{401}\0\u{410}\u{423}\0\u{425}\u{44f}\u{451}",
        30,
    ),
    glyph_data: include_bytes!("glyph_data.raw"),
    ligature_code_points: "\0\u{66}\u{66}\u{69}\0\u{66}\u{66}\0\u{66}\u{69}\0\u{66}\u{6a}\0\u{67}\u{6a}\0\u{6a}\u{6a}\0\u{73}\u{73}\0\u{79}\u{6a}",
};
