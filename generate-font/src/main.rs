use clap::{Parser, ValueEnum};
use image::{codecs::png::PngEncoder, ImageEncoder};
use std::{collections::BTreeMap, convert::TryFrom, error::Error, fs::File, io::Write, path::Path};
mod elm_file_data;
use elm_file_data::ElmFileData;
mod glyph_images;
use glyph_images::{CodePoint, GlyphImages};

const ATLAS_WIDTH: u32 = 128;

#[derive(ValueEnum, Clone, Default, Debug)]
enum Charset {
    #[default]
    ASCII,
    All,
}

impl std::fmt::Display for Charset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Charset::ASCII => write!(f, "ascii"),
            Charset::All => write!(f, "all"),
        }
    }
}

// Clapp application parameters
#[derive(Parser)]
struct GenerateFont {
    /// Path to the font files
    /// Default: "mogeefont/font"
    #[clap(long, default_value = "mogeefont/font")]
    font_dir: String,

    /// Path to the Elm module
    #[clap(long, default_value = "mogeefont/src/MogeeFont.elm")]
    elm_file: String,

    /// Path to the output directory
    /// Default: "src"
    #[clap(long, default_value = "src")]
    out_dir: String,

    /// Character subsetting
    /// Default: "ascii"
    #[clap(long, value_enum, default_value = "ascii")]
    charset: Vec<Charset>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = GenerateFont::parse();
    let glyph_images = GlyphImages::try_from(args.font_dir.as_ref())?;
    let elm_file_data = ElmFileData::try_from(args.elm_file.as_ref())?;
    let mut rust_file = std::fs::File::create(&Path::new(&args.out_dir).join("generated.rs"))?;

    write!(
        &mut rust_file,
        r#"use crate::{{
    charset::Charset, kerning::Kerning, ligatures::Ligatures, side_bearings::SideBearings
}};
use embedded_graphics::{{image::ImageRaw, mono_font::mapping::StrGlyphMapping}};
"#
    )?;

    let fonts: Vec<_> = args
        .charset
        .into_iter()
        .map(|charset| FontData::new(&glyph_images, &elm_file_data, charset))
        .collect();

    for font in fonts.iter() {
        font.write(
            &mut rust_file,
            &Path::new(&args.out_dir).join("lib.rs"),
            &Path::new(&args.out_dir).join(format!("{}_font.raw", font.charset)),
            &Path::new(&args.out_dir).join(format!("{}_glyph_data.raw", font.charset)),
        )?;
    }

    update_specimen(&Path::new(&args.out_dir).join("lib.rs"), &fonts)?;

    Ok(())
}

#[derive(Debug)]
struct Glyph {
    left: u32,
    top: u32,
    left_kerning_class: u8,
    right_kerning_class: u8,
    img: image::GrayImage,
}

struct FontData {
    charset: Charset,
    glyph_code_points: Vec<u32>,
    ligature_code_points: Vec<Vec<u32>>,
    glyphs: Vec<Glyph>,
    atlas_height: u32,
    line_height: u32,
    glyph_bearings: Vec<(u32, i8, i8)>,
    default_bearings: (i8, i8),
    kerning_overrides: Vec<(u32, u32, i8)>,
    kering_pairs: Vec<(u8, u8, i8)>,
}

impl FontData {
    pub fn new(glyphs_images: &GlyphImages, elm_file_data: &ElmFileData, charset: Charset) -> Self {
        let GlyphImages {
            mut code_points_and_images,
        } = glyphs_images.clone();

        let ElmFileData {
            line_height,
            space_width,
            default_bearings,
            mut bearings,
            mut left_kerning_class,
            mut right_kerning_class,
            mut kering_pairs,
            mut kerning_overrides,
        } = elm_file_data.clone();

        let mut excluded = Vec::new();

        // Filter out non-ASCII characters
        if let Charset::ASCII = charset {
            (code_points_and_images, excluded) =
                code_points_and_images
                    .into_iter()
                    .partition(|(code_point, _)| match code_point {
                        CodePoint::Single(p) => (*p as u32) < 128,
                        CodePoint::Ligature(p) => p.chars().all(|c| (c as u32) < 128),
                    });

            bearings = bearings
                .into_iter()
                .filter(|(code_point, _)| code_point.chars().all(|c| (c as u32) < 128))
                .collect();
            left_kerning_class = left_kerning_class
                .into_iter()
                .map(|(c, chars)| {
                    (
                        c,
                        chars
                            .into_iter()
                            .filter(|code_point| code_point.chars().all(|c| (c as u32) < 128))
                            .collect(),
                    )
                })
                .collect();
            right_kerning_class = right_kerning_class
                .into_iter()
                .map(|(c, chars)| {
                    (
                        c,
                        chars
                            .into_iter()
                            .filter(|code_point| code_point.chars().all(|c| (c as u32) < 128))
                            .collect(),
                    )
                })
                .collect();
            kering_pairs.retain(|(left_cls, right_cls, _)| {
                left_kerning_class.iter().any(|(cls, _)| cls == left_cls)
                    && right_kerning_class.iter().any(|(cls, _)| cls == right_cls)
            });

            kerning_overrides.retain(|(left, right, _)| {
                left.chars().all(|c| (c as u32) < 128) && right.chars().all(|c| (c as u32) < 128)
            });
        }

        println!(
            "Included characters: {{ {} }}",
            code_points_and_images
                .iter()
                .map(|(c, _)| c)
                .map(|c| format!("{}", c))
                .collect::<Vec<String>>()
                .join(", ")
        );

        println!(
            "Excluded characters: {{ {} }}",
            excluded
                .iter()
                .map(|(c, _)| c)
                .map(|c| format!("{}", c))
                .collect::<Vec<String>>()
                .join(", ")
        );

        // Ensure we can use binary search on the kerning pairs
        kering_pairs.sort_by_key(|(left, right, _)| (*left, *right));

        // Add a space glyph
        code_points_and_images.push((
            CodePoint::Single(' '),
            image::GrayImage::from_pixel(space_width, line_height, image::Luma::from([255])),
        ));

        // First glyphs, then ligatures
        code_points_and_images.sort_by(|a, b| a.0.cmp(&b.0));

        let mut glyph_code_points = Vec::new();
        let mut ligature_code_points = Vec::new();
        let mut glyph_bearings = Vec::new();
        let mut code_point_to_offset = BTreeMap::new();
        for (glyph_offset, (code_point, _)) in code_points_and_images.iter().enumerate() {
            let str_code_point = code_point.as_string();

            code_point_to_offset.insert(str_code_point.clone(), glyph_offset as u32);

            if let Some((left_bearing, right_bearing)) = bearings.get(&str_code_point) {
                glyph_bearings.push((glyph_offset as u32, *left_bearing, *right_bearing));
            }

            match code_point {
                CodePoint::Single(p) => {
                    glyph_code_points.push(*p as u32);
                }
                CodePoint::Ligature(p) => {
                    ligature_code_points.push(p.chars().map(|c| c.into()).collect());
                }
            }
        }

        // Verify that we have all the printable ASCII characters:
        if let Charset::ASCII = charset {
            let missing_glyphs: Vec<u32> = (32..127)
                .filter(|i| !glyph_code_points.contains(i))
                .collect();
            if !missing_glyphs.is_empty() {
                panic!(
                    "Missing ASCII characters: {{ {} }}",
                    missing_glyphs
                        .iter()
                        .map(|i| format!(
                            "{:0>4X}: \"{}\"",
                            i,
                            char::from_u32(*i).unwrap_or_default()
                        ))
                        .collect::<Vec<String>>()
                        .join(", ")
                );
            }
        }

        let mut left_kerning_classes: BTreeMap<String, u8> = BTreeMap::new();
        for (class, chars) in left_kerning_class {
            for c in chars {
                left_kerning_classes.insert(c, class);
            }
        }
        let mut right_kerning_classes: BTreeMap<String, u8> = BTreeMap::new();
        for (class, chars) in right_kerning_class {
            for c in chars {
                right_kerning_classes.insert(c, class);
            }
        }

        let mut kerning_overrides = kerning_overrides
            .into_iter()
            .map(|(left_char, right_char, kerning)| {
                (
                    code_point_to_offset[&left_char],
                    code_point_to_offset[&right_char],
                    kerning,
                )
            })
            .collect::<Vec<_>>();

        // Ensure we can use binary search on the kerning overrides
        kerning_overrides.sort_by_key(|(left, right, _)| (*left, *right));

        let mut left = 0;
        let mut top = 0;
        let mut glyphs = Vec::new();
        // 1 pixel spacing between glyphs is for pure aesthetic reasons
        let spacing = 1;
        for (code_point, img) in code_points_and_images.into_iter() {
            let width = img.width();
            if left + width > ATLAS_WIDTH {
                left = 0;
                top += line_height + spacing;
            }
            glyphs.push(Glyph {
                left,
                top,
                left_kerning_class: *left_kerning_classes
                    .get(&code_point.as_string())
                    .unwrap_or(&0),
                right_kerning_class: *right_kerning_classes
                    .get(&code_point.as_string())
                    .unwrap_or(&0),
                img: img.clone(),
            });
            left += width + spacing;
        }
        let atlas_height = top + line_height;

        Self {
            charset,
            glyphs,
            glyph_code_points,
            ligature_code_points,
            atlas_height,
            line_height,
            glyph_bearings,
            default_bearings,
            kerning_overrides,
            kering_pairs,
        }
    }

    fn bitmap_data(&self) -> Result<Vec<u8>, std::num::TryFromIntError> {
        let bitmap_size = usize::try_from(ATLAS_WIDTH * self.atlas_height)?;
        let mut bitmap = vec![false; bitmap_size];
        for glyph in self.glyphs.iter() {
            for y in 0..glyph.img.height() {
                for x in 0..glyph.img.width() {
                    if glyph.img.get_pixel(x, y).0[0] == 0 {
                        let index =
                            usize::try_from(glyph.left + x + (glyph.top + y) * ATLAS_WIDTH)?;
                        bitmap[index] = true;
                    }
                }
            }
        }
        Ok(bitmap
            .chunks_exact(8)
            .map(|byte| {
                byte.iter()
                    .enumerate()
                    .filter(|(_, bit)| **bit)
                    .map(|(i, _)| 0x80 >> i)
                    .sum()
            })
            .collect())
    }

    pub fn png_data(&self, scale: u32) -> Result<String, Box<dyn Error>> {
        let mut png = Vec::new();
        let bitmap_data = self.bitmap_data()?;
        let image =
            image::GrayImage::from_fn(ATLAS_WIDTH * scale, self.atlas_height * scale, |x, y| {
                let x = x / scale;
                let y = y / scale;
                let index = usize::try_from(x / 8 + y * (ATLAS_WIDTH / 8)).unwrap_or_default();
                let bit = bitmap_data[index] & (128 >> x % 8) != 0;
                image::Luma::from([(if bit { 255 } else { 0 })])
            });
        let width = image.width();
        let height = image.height();
        let data = image.into_raw();
        PngEncoder::new(&mut png).write_image(&data, width, height, image::ColorType::L8.into())?;
        Ok(format!("data:image/png;base64,{}", &base64::encode(&png)))
    }

    /// Generate a string representation of the glyph mapping
    /// and the substitute character index
    fn glyph_mapping(&self) -> (String, usize) {
        let mut st = String::new();
        // group codepoints in ranges of subsequent codepoints
        let mut start = self.glyph_code_points[0];
        let mut last = self.glyph_code_points[0];
        let mut substitute_index = 0;
        for (i, &code_point) in self.glyph_code_points.iter().skip(1).enumerate() {
            // if the code point is '?' then we remember the index
            if code_point == '?'.into() {
                substitute_index = i + 1;
            }
            if code_point == last + 1 {
                last = code_point;
            } else {
                if start == last {
                    st.push_str(&format!("\\u{{{:x}}}", start));
                } else {
                    st.push_str(&format!("\\0\\u{{{:x}}}\\u{{{:x}}}", start, last));
                }
                start = code_point;
                last = code_point;
            }
        }
        if start == last {
            st.push_str(&format!("\\u{{{:x}}}", start));
        } else {
            st.push_str(&format!("\\0\\u{{{:x}}}\\u{{{:x}}}", start, last));
        }
        (st, substitute_index)
    }

    /// Generate a string representation of the side bearings
    fn side_bearings(&self) -> String {
        let mut st = String::new();
        for (i, (code_point, left_bearing, right_bearing)) in self.glyph_bearings.iter().enumerate()
        {
            if i > 0 {
                st.push_str(", ");
            }
            st.push_str(&format!(
                "({}, {}, {})",
                code_point, left_bearing, right_bearing
            ));
        }
        st
    }

    /// Generate a string representation of the kerning pairs
    fn kerning_pairs(&self) -> String {
        let mut st = String::new();
        for (i, (left_class, right_class, kerning)) in self.kering_pairs.iter().enumerate() {
            if i > 0 {
                st.push_str(", ");
            }
            st.push_str(&format!("({}, {}, {})", left_class, right_class, kerning));
        }
        st
    }

    /// Generate a string representation of the kerning overrides
    fn kerning_overrides(&self) -> String {
        let mut st = String::new();
        for (i, (left, right, kerning)) in self.kerning_overrides.iter().enumerate() {
            if i > 0 {
                st.push_str(", ");
            }
            st.push_str(&format!("({}, {}, {})", left, right, kerning));
        }
        st
    }

    /// Generate a string representation of the ligature code points
    fn ligature_code_points(&self) -> String {
        let mut st = String::new();
        for code_points in self.ligature_code_points.iter() {
            st.push_str("\\0");
            for code_point in code_points {
                st.push_str(&format!("\\u{{{:x}}}", code_point));
            }
        }
        st
    }

    fn save_raw_font<P: AsRef<Path>>(&self, raw_file: &P) -> Result<(), Box<dyn Error>> {
        std::fs::write(raw_file, &self.bitmap_data()?)?;
        Ok(())
    }

    fn save_raw_glyph_data<P: AsRef<Path>>(&self, raw_file: &P) -> std::io::Result<()> {
        let mut glyph_data = Vec::new();
        for glyph in self.glyphs.iter() {
            // concat width and height into a single u8
            let dimensions = (glyph.img.height() as u8) << 4 | (glyph.img.width() as u8);
            glyph_data.extend_from_slice(&[
                glyph.left as u8,
                glyph.top as u8,
                dimensions,
                glyph.left_kerning_class,
                glyph.right_kerning_class,
            ]);
        }
        std::fs::write(raw_file, &glyph_data)
    }

    pub fn write<P: AsRef<Path>>(
        &self,
        file: &mut File,
        rust_file: &P,
        raw_file: &P,
        raw_glyphs_file: &P,
    ) -> Result<(), Box<dyn Error>> {
        self.save_raw_font(raw_file)?;
        self.save_raw_glyph_data(raw_glyphs_file)?;

        let relative_raw_path = raw_file
            .as_ref()
            .strip_prefix(rust_file.as_ref().parent().unwrap())?
            .to_str()
            .unwrap_or_default();

        let relative_glyphs_path = raw_glyphs_file
            .as_ref()
            .strip_prefix(rust_file.as_ref().parent().unwrap())?
            .to_str()
            .unwrap_or_default();

        let (glyph_mapping, substitute_index) = self.glyph_mapping();
        let ligature_code_points = self.ligature_code_points();
        let ligature_offset = self.glyph_code_points.len();

        let png_data = self.png_data(2)?;

        let line_height = self.line_height;
        let side_bearings = self.side_bearings();
        let default_bearings =
            format!("({}, {})", self.default_bearings.0, self.default_bearings.1);

        let kerning_pairs = self.kerning_pairs();
        let kerning_overrides = self.kerning_overrides();

        let charset_upper = format!("{}", self.charset).to_uppercase();

        writeln!(
            file,
            r#"
/// {charset_upper} charset
///
/// ![specimen]({png_data})
pub const {charset_upper}: Charset = Charset {{
    image: ImageRaw::new(include_bytes!("{relative_raw_path}"), {ATLAS_WIDTH}),
    glyph_mapping: StrGlyphMapping::new(
        "{glyph_mapping}",
        {substitute_index},
    ),
    glyph_data: include_bytes!("{relative_glyphs_path}"),
    side_bearings: SideBearings::new(
        &[{side_bearings}],
        {default_bearings},
    ),
    kerning: Kerning::new(
        &[{kerning_pairs}],
        &[{kerning_overrides}],
    ),
    ligatures: Ligatures::new(
        "{ligature_code_points}",
        {ligature_offset},
    ),
    line_height: {line_height},
    baseline: 8,
}};"#,
        )?;

        Ok(())
    }
}

fn update_specimen<P: AsRef<Path>>(file: &P, fonts: &[FontData]) -> Result<(), Box<dyn Error>> {
    let input = std::fs::read_to_string(file)?;
    let mut output = Vec::new();
    let mut take = true;
    for line in input.lines() {
        if take {
            output.push(line.to_string());
        }
        if line.trim() == "//START-SPECIMEN" {
            take = false;
            output.push("//! | Charset | Specimen, upscaled to 2x |".to_string());
            output.push("//! |---------|----------|".to_string());
            for font in fonts {
                output.push(format!(
                    "//! | `{charset}` | ![{charset}]({png_data}) |",
                    charset = font.charset.to_string().to_uppercase(),
                    png_data = font.png_data(2)?,
                ));
            }
        }
        if line.trim() == "//END-SPECIMEN" {
            output.push(line.to_string());
            take = true;
        }
    }
    output.push(String::new());
    std::fs::write(file, output.join("\n"))?;
    Ok(())
}
