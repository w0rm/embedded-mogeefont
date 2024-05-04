use crate::{font::Font, kerning::Kerning, ligatures::Ligatures, side_bearings::SideBearings};
use embedded_graphics::{image::ImageRaw, mono_font::mapping::StrGlyphMapping};

/// ![mogeefont](data:image/png;base64,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)
pub const MOGEEFONT: Font<'_> = Font {
    image: ImageRaw::new(include_bytes!("mogeefont.raw"), 128),
    glyph_mapping: StrGlyphMapping::new(
        "\0\u{20}\u{7e}",
        31,
    ),
    glyph_data: include_bytes!("glyph_data.raw"),
    side_bearings: SideBearings::new(
        &[(0, 0, 0), (12, 0, 1), (27, 0, 1), (74, -2, 1), (100, -2, 1)],
        (0, 1),
    ),
    kerning: Kerning::new(
        &[(1, 14, -1), (2, 2, -1), (2, 6, 0), (2, 10, -1), (2, 13, -1), (2, 14, -1), (3, 3, -1), (3, 7, -1), (3, 14, -1), (4, 2, -2), (4, 4, -1), (4, 6, -1), (4, 7, -1), (4, 10, -1), (4, 13, -2), (4, 14, -2), (5, 3, -1), (5, 14, -1), (6, 2, -2), (6, 3, -1), (6, 4, -1), (6, 5, -1), (6, 6, -1), (6, 7, -2), (6, 13, -2), (6, 14, -2), (7, 3, -1), (7, 7, 0), (8, 7, 0), (13, 2, -1), (13, 14, -1), (14, 2, -1), (14, 6, -1), (14, 10, -1), (14, 14, -1), (15, 2, -1), (15, 13, -1), (16, 3, -1), (16, 7, -1), (16, 9, -1), (16, 14, -1), (17, 14, -1)],
        &[(15, 15, -2), (35, 70, -1), (41, 70, -1), (60, 60, -2), (70, 52, -2), (81, 70, -1), (81, 95, -1), (81, 96, -1), (81, 97, -1)],
    ),
    ligatures: Ligatures::new(
        "\0\u{66}\u{66}\u{69}\0\u{66}\u{66}\0\u{66}\u{69}\0\u{66}\u{6a}\0\u{67}\u{6a}\0\u{6a}\u{6a}\0\u{73}\u{73}\0\u{79}\u{6a}",
        95,
    ),
    em_height: 11,
    baseline: 8,
};
