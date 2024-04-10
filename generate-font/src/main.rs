use clap::Parser;
use image::imageops::resize;
use image::{codecs::png::PngEncoder, ImageEncoder, ImageResult};
use std::{collections::BTreeMap, convert::TryFrom, fmt::Write, io::Write as IOWrite, path::Path};
mod elm_file_data;
use elm_file_data::ElmFileData;

mod glyph_images;
use glyph_images::{CodePoint, GlyphImages};

const ATLAS_WIDTH: u32 = 128;

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

    /// Path to the output png file
    /// Default: "assets/mogeefont.png"
    #[clap(long, default_value = "assets/mogeefont.png")]
    png_file: String,

    /// Path to the output raw file
    /// Default: "src/mogeefont.raw"
    #[clap(long, default_value = "src/mogeefont.raw")]
    raw_file: String,

    /// Path to the output raw file
    /// Default: "src/glyph_data.raw"
    #[clap(long, default_value = "src/glyph_data.raw")]
    raw_glyph_data: String,

    /// Path to the output rust file
    /// Default: "src/generated.rs"
    #[clap(long, default_value = "src/generated.rs")]
    rust_file: String,
}

fn main() {
    let args = GenerateFont::parse();
    let glyph_images = GlyphImages::try_from(args.font_dir.as_ref()).unwrap();
    let elm_file_data = ElmFileData::try_from(args.elm_file.as_ref()).unwrap();
    let font_data = FontData::new(glyph_images, elm_file_data);

    font_data.save_png(&args.png_file, 2).unwrap();
    font_data.save_raw(&args.raw_file).unwrap();
    font_data.save_raw_glyph_data(&args.raw_glyph_data).unwrap();
    font_data
        .save_rust(&args.rust_file, &args.raw_file, &args.raw_glyph_data)
        .unwrap();
}

#[derive(Debug)]
struct Glyph {
    left: u32,
    top: u32,
    width: u32,
    height: u32,
    img: image::GrayImage,
}

struct FontData {
    bitmap_data: Vec<u8>,
    glyph_code_points: Vec<u32>,
    ligature_code_points: Vec<Vec<u32>>,
    glyphs: Vec<Glyph>,
    atlas_height: u32,
    em_height: u32,
}

impl FontData {
    fn new(glyphs_images: GlyphImages, elm_file_data: ElmFileData) -> Self {
        let mut code_points_and_images = glyphs_images.glyphs;
        let em_height = elm_file_data.em_height;

        // Add a space glyph
        code_points_and_images.push((
            CodePoint::Single(" ".to_string()),
            image::GrayImage::from_pixel(
                elm_file_data.space_width,
                em_height,
                image::Luma::from([255]),
            ),
        ));

        // First glyphs, then ligatures
        code_points_and_images.sort_by(|a, b| a.0.cmp(&b.0));

        let (code_points, images) = code_points_and_images
            .into_iter()
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let mut glyph_code_points = Vec::new();
        let mut ligature_code_points = Vec::new();
        let mut glyph_offsets: BTreeMap<&str, usize> = BTreeMap::new();
        for (glyph_offset, code_point) in code_points.iter().enumerate() {
            match code_point {
                CodePoint::Single(p) => {
                    glyph_code_points.push(p.chars().next().unwrap() as u32);
                    glyph_offsets.insert(&p, glyph_offset);
                }
                CodePoint::Ligature(p) => {
                    ligature_code_points.push(p.chars().map(|c| c as u32).collect());
                    glyph_offsets.insert(&p, glyph_offset);
                }
            }
        }

        let mut left = 0;
        let mut top = 0;
        let mut glyphs = Vec::new();
        for img in images {
            let (width, height) = img.dimensions();
            if left + width > ATLAS_WIDTH {
                left = 0;
                top += em_height + 1;
            }
            glyphs.push(Glyph {
                left,
                top,
                width,
                height,
                img: img.clone(),
            });
            left += width + 1;
        }

        let atlas_height = top + em_height;

        let bitmap_size = usize::try_from(ATLAS_WIDTH * atlas_height).unwrap();
        let mut bitmap = vec![false; bitmap_size];

        for glyph in glyphs.iter() {
            for y in 0..glyph.height {
                for x in 0..glyph.width {
                    if glyph.img.get_pixel(x, y).0[0] == 0 {
                        let index = usize::try_from(glyph.left + x + (glyph.top + y) * ATLAS_WIDTH)
                            .unwrap();
                        bitmap[index] = true;
                    }
                }
            }
        }

        let bitmap_data = bitmap
            .chunks_exact(8)
            .map(|byte| {
                byte.iter()
                    .enumerate()
                    .filter(|(_, bit)| **bit)
                    .map(|(i, _)| 0x80 >> i)
                    .sum()
            })
            .collect();

        Self {
            bitmap_data,
            glyphs,
            glyph_code_points,
            ligature_code_points,
            atlas_height,
            em_height,
        }
    }

    pub fn png_data(&self, scale: u32) -> String {
        let mut png = Vec::new();
        let image = self.scaled_image(scale);
        let width = image.width();
        let height = image.height();
        let data = image.into_raw();
        PngEncoder::new(&mut png)
            .write_image(&data, width, height, image::ColorType::L8.into())
            .unwrap();
        format!("data:image/png;base64,{}", &base64::encode(&png))
    }

    fn scaled_image(&self, scale: u32) -> image::GrayImage {
        let image = image::GrayImage::from_fn(ATLAS_WIDTH, self.atlas_height, |x, y| {
            let index = usize::try_from(x / 8 + y * (ATLAS_WIDTH / 8)).unwrap();
            image::Luma::from([(self.bitmap_data[index] & (128 >> x % 8)) * 255])
        });
        resize(
            &image,
            image.width() * scale,
            image.height() * scale,
            image::imageops::FilterType::Nearest,
        )
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
            if code_point == '?' as u32 {
                substitute_index = i + 1;
            }
            if code_point == last + 1 {
                last = code_point;
            } else {
                if start == last {
                    write!(st, "\\u{{{:x}}}", start).unwrap();
                } else {
                    write!(st, "\\0\\u{{{:x}}}\\u{{{:x}}}", start, last).unwrap();
                }
                start = code_point;
                last = code_point;
            }
        }
        if start == last {
            write!(st, "\\u{{{:x}}}", start).unwrap();
        } else {
            write!(st, "\\0\\u{{{:x}}}\\u{{{:x}}}", start, last).unwrap();
        }
        (st, substitute_index)
    }

    fn ligature_code_points(&self) -> String {
        let mut st = String::new();
        for code_points in self.ligature_code_points.iter() {
            write!(st, "\\0").unwrap();
            for code_point in code_points {
                write!(st, "\\u{{{:x}}}", code_point).unwrap();
            }
        }
        st
    }

    pub fn save_png<P: AsRef<Path>>(&self, png_file: &P, scale: u32) -> ImageResult<()> {
        self.scaled_image(scale).save(png_file)
    }

    pub fn save_raw<P: AsRef<Path>>(&self, raw_file: &P) -> std::io::Result<()> {
        std::fs::write(raw_file, &self.bitmap_data)
    }

    pub fn save_raw_glyph_data<P: AsRef<Path>>(&self, raw_file: &P) -> std::io::Result<()> {
        let mut glyph_data = Vec::new();
        for glyph in self.glyphs.iter() {
            glyph_data.extend_from_slice(&[
                glyph.left as u8,
                glyph.top as u8,
                glyph.width as u8,
                glyph.height as u8,
            ]);
        }
        std::fs::write(raw_file, &glyph_data)
    }

    pub fn save_rust<P: AsRef<Path>>(
        &self,
        rust_file: &P,
        raw_file: &P,
        raw_glyphs_file: &P,
    ) -> std::io::Result<()> {
        let relative_raw_path = raw_file
            .as_ref()
            .strip_prefix(rust_file.as_ref().parent().unwrap())
            .unwrap()
            .to_str()
            .unwrap();

        let relative_glyphs_path = raw_glyphs_file
            .as_ref()
            .strip_prefix(rust_file.as_ref().parent().unwrap())
            .unwrap()
            .to_str()
            .unwrap();

        let (glyph_mapping, substitute_index) = self.glyph_mapping();
        let ligature_code_points = self.ligature_code_points();
        let ligature_offset = self.glyph_code_points.len();

        let png_data = self.png_data(2);

        let mut file = std::fs::File::create(rust_file)?;

        let character_height = self.em_height;

        writeln!(
            file,
            r#"use crate::font::Font;
use crate::ligature_substitution::StrLigatureSubstitution;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::mono_font::mapping::StrGlyphMapping;

/// ![mogeefont]({png_data})
pub const MOGEEFONT: Font<'_> = Font {{
    image: ImageRaw::new(include_bytes!("{relative_raw_path}"), 128),
    glyph_mapping: StrGlyphMapping::new(
        "{glyph_mapping}",
        {substitute_index},
    ),
    glyph_data: include_bytes!("{relative_glyphs_path}"),
    ligature_substitution: StrLigatureSubstitution::new(
        "{ligature_code_points}",
        {ligature_offset},
    ),
    character_height: {character_height},
    baseline: 8,
    character_spacing: 1,
}};"#,
        )
    }
}
