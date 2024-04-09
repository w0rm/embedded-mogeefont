use clap::Parser;
use image::imageops::resize;
use image::{codecs::png::PngEncoder, ImageEncoder, ImageResult};
use std::{collections::BTreeMap, convert::TryFrom, fmt::Write, io::Write as IOWrite, path::Path};

mod elm_file_data;
use elm_file_data::ElmFileData;

const ATLAS_WIDTH: usize = 128;

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
    let font_data = FontData::try_from((args.font_dir, args.elm_file)).unwrap();
    font_data.save_png(&args.png_file, 2).unwrap();
    font_data.save_raw(&args.raw_file).unwrap();
    font_data.save_raw_glyph_data(&args.raw_glyph_data).unwrap();

    font_data
        .save_rust(&args.rust_file, &args.raw_file, &args.raw_glyph_data)
        .unwrap();
}

#[derive(Debug, PartialEq, Eq)]
enum CodePoint {
    Single(String),
    Ligature(String),
}

impl Ord for CodePoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for CodePoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (CodePoint::Single(a), CodePoint::Single(b)) => a.partial_cmp(b),
            (CodePoint::Ligature(a), CodePoint::Ligature(b)) => {
                if a.len() < b.len() {
                    // longer ligatures should come first
                    // because we want fff to be before ff
                    Some(std::cmp::Ordering::Greater)
                } else if a.len() > b.len() {
                    Some(std::cmp::Ordering::Less)
                } else {
                    a.partial_cmp(b)
                }
            }
            (CodePoint::Single(_), CodePoint::Ligature(_)) => Some(std::cmp::Ordering::Less),
            (CodePoint::Ligature(_), CodePoint::Single(_)) => Some(std::cmp::Ordering::Greater),
        }
    }
}

#[derive(Debug)]
struct Glyph {
    code_point: CodePoint,
    left: usize,
    top: usize,
    img: image::GrayImage,
}

struct FontData {
    bitmap_data: Vec<u8>,
    glyph_code_points: Vec<u32>,
    ligature_code_points: Vec<Vec<u32>>,
    glyph_data: Vec<u8>, // left, top, width, height for each glyph then ligature
    height: usize,
    em_height: u32,
}

impl FontData {
    fn new(glyphs_data: Vec<Glyph>, height: usize, elm_file_data: ElmFileData) -> Self {
        let mut bitmap = vec![false; ATLAS_WIDTH * height];

        let mut glyph_code_points = Vec::new();
        let mut ligature_code_points = Vec::new();
        let mut glyph_data = Vec::new();

        // mapping from glyph code points to glyph index
        let mut glyph_offsets: BTreeMap<&str, usize> = BTreeMap::new();

        for (glyph_offset, glyph) in glyphs_data.iter().enumerate() {
            let glyph_width = usize::try_from(glyph.img.width()).unwrap();
            let glyph_height = usize::try_from(glyph.img.height()).unwrap();
            for y in 0..glyph_height {
                for x in 0..glyph_width {
                    if glyph.img.get_pixel(x as u32, y as u32).0[0] == 0 {
                        bitmap[glyph.left + x + (glyph.top + y) * ATLAS_WIDTH] = true;
                    }
                }
            }

            glyph_data.extend_from_slice(&[
                glyph.left as u8,
                glyph.top as u8,
                glyph_width as u8,
                glyph_height as u8,
            ]);

            match &glyph.code_point {
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

        let bitmap_data = bitmap
            .chunks_exact(8)
            .map(|byte| {
                byte.iter()
                    .enumerate()
                    .filter(|(_, bit)| **bit)
                    .map(|(i, _)| 0x80 >> i)
                    .sum()
            })
            .collect::<Vec<_>>();

        Self {
            bitmap_data,
            glyph_data,
            glyph_code_points,
            ligature_code_points,
            height,
            em_height: elm_file_data.em_height,
        }
    }

    fn pixel(&self, x: usize, y: usize) -> bool {
        self.bitmap_data[x / 8 + y * (ATLAS_WIDTH / 8)] & (128 >> x % 8) != 0
    }

    fn to_png(&self) -> image::GrayImage {
        let mut img = image::GrayImage::new(ATLAS_WIDTH as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..ATLAS_WIDTH {
                img.put_pixel(
                    x as u32,
                    y as u32,
                    image::Luma::from([255 * self.pixel(x, y) as u8]),
                );
            }
        }
        img
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
        let image = self.to_png();
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
        std::fs::write(raw_file, &self.glyph_data)
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

impl<P> TryFrom<(P, P)> for FontData
where
    P: AsRef<Path>,
{
    type Error = std::io::Error;

    fn try_from(paths: (P, P)) -> std::io::Result<Self> {
        // Read the png images from the font directory
        let font_dir = std::fs::read_dir(paths.0)?;
        let elm_file_data = ElmFileData::from(paths.1);

        let mut all_glyphs: Vec<(CodePoint, image::GrayImage)> = Vec::new();

        // Iterate over the png images in the font directory
        for entry in font_dir {
            let entry = match entry {
                Ok(entry) => match entry.path().extension() {
                    Some(ext) if ext != "png" => continue,
                    None => continue,
                    _ => entry,
                },
                Err(_) => continue,
            };

            // Extract the unicode code points from the file stem
            // for ligatures there are multiple code points, separated by "_"
            let path = entry.path();
            let img = image::open(&path).unwrap().to_luma8();
            let code_points: Vec<char> = path
                .file_stem()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .split('_')
                .map(|s| char::from_u32(u32::from_str_radix(s, 16).unwrap()).unwrap())
                .collect();

            let code_point = match code_points.len() {
                0 => panic!("No code points found"),
                1 => CodePoint::Single(code_points.into_iter().collect()),
                _ => CodePoint::Ligature(code_points.into_iter().collect()),
            };

            all_glyphs.push((code_point, img));
        }

        // Add a space glyph
        all_glyphs.push((
            CodePoint::Single(" ".to_string()),
            image::GrayImage::from_pixel(
                elm_file_data.space_width,
                elm_file_data.em_height,
                image::Luma::from([255]),
            ),
        ));

        // First glyphs, then ligatures
        all_glyphs.sort_by(|a, b| a.0.cmp(&b.0));

        let mut left = 0;
        let mut top = 0;
        let mut glyphs_data = Vec::new();

        for (code_point, img) in all_glyphs {
            let width = usize::try_from(img.width()).unwrap();
            if left + width > ATLAS_WIDTH {
                left = 0;
                top += elm_file_data.em_height as usize + 1;
            }
            glyphs_data.push(Glyph {
                left,
                top,
                img,
                code_point,
            });
            left += width + 1;
        }

        Ok(FontData::new(
            glyphs_data,
            top + elm_file_data.em_height as usize,
            elm_file_data,
        ))
    }
}
