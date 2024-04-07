use clap::Parser;
use image::imageops::resize;
use image::{codecs::png::PngEncoder, ImageEncoder, ImageResult};
use std::convert::TryFrom;
use std::fs;
use std::io::Write;
use std::path::Path;

const ATLAS_WIDTH: usize = 128;
const GLYPH_HEIGHT: usize = 11;

// Clapp application parameters
#[derive(Parser)]
struct GenerateFont {
    /// Path to the font files
    /// Default: "mogeefont/font"
    #[clap(long, default_value = "mogeefont/font")]
    font_dir: String,

    /// Path to the output png file
    /// Default: "assets/mogeefont.png"
    #[clap(long, default_value = "assets/mogeefont.png")]
    png_file: String,

    /// Path to the output raw file
    /// Default: "src/mogeefont.raw"
    #[clap(long, default_value = "src/mogeefont.raw")]
    raw_file: String,

    /// Path to the output rust file
    /// Default: "src/mogeefont.rs"
    #[clap(long, default_value = "src/mogeefont.rs")]
    rust_file: String,
}

fn main() {
    let args = GenerateFont::parse();
    let font_data = FontData::try_from(InputWrapper(args.font_dir)).unwrap();
    font_data.save_png(&args.png_file, 2).unwrap();
    font_data.save_raw(&args.raw_file).unwrap();
    font_data
        .save_rust(&args.rust_file, &args.raw_file)
        .unwrap();
}

#[derive(Debug, PartialEq, Eq)]
enum CodePoint {
    Single(u16),
    Ligature(Vec<u16>),
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
    glyph_code_points: Vec<u16>,
    ligature_code_points: Vec<Vec<u16>>,
    glyph_data: Vec<u8>, // left, top, width, height for each glyph then ligature
    height: usize,
}

impl FontData {
    fn new(glyphs_data: Vec<Glyph>, height: usize) -> Self {
        let mut bitmap = vec![false; ATLAS_WIDTH * height];

        let mut glyph_code_points = Vec::new();
        let mut ligature_code_points = Vec::new();
        let mut glyph_data = Vec::new();

        for glyph in glyphs_data {
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

            match glyph.code_point {
                CodePoint::Single(p) => {
                    glyph_code_points.push(p);
                }
                CodePoint::Ligature(p) => {
                    ligature_code_points.push(p);
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

    pub fn save_png<P: AsRef<Path>>(&self, png_file: &P, scale: u32) -> ImageResult<()> {
        self.scaled_image(scale).save(png_file)
    }

    pub fn save_raw<P: AsRef<Path>>(&self, raw_file: &P) -> std::io::Result<()> {
        fs::write(raw_file, &self.bitmap_data)
    }

    pub fn save_rust<P: AsRef<Path>>(&self, rust_file: &P, raw_file: &P) -> std::io::Result<()> {
        let relative_path = raw_file
            .as_ref()
            .strip_prefix(rust_file.as_ref().parent().unwrap())
            .unwrap();

        let mut file = fs::File::create(rust_file)?;
        writeln!(file, "use embedded_graphics::image::ImageRaw;")?;
        writeln!(
            file,
            "use embedded_graphics::mono_font::mapping::StrGlyphMapping;"
        )?;
        writeln!(file, "use embedded_graphics::pixelcolor::BinaryColor;\n")?;

        writeln!(
            file,
            r#"pub const IMAGE: ImageRaw<'_, BinaryColor> = ImageRaw::new(include_bytes!("{}"), {});"#,
            relative_path.to_str().unwrap(),
            ATLAS_WIDTH
        )?;
        writeln!(
            file,
            "pub const MOGEEFONT_GLYPH_DATA: [u8; {}] = [",
            self.glyph_data.len()
        )?;
        for (i, byte) in self.glyph_data.iter().enumerate() {
            if i % 16 == 0 {
                write!(file, "    ")?;
            }
            write!(file, "{:#04x},", byte)?;
            if i % 16 == 15 {
                writeln!(file)?;
            } else if i < self.glyph_data.len() - 1 {
                write!(file, " ")?;
            }
        }
        writeln!(file, "\n];")?;

        writeln!(
            file,
            "pub const MOGEEFONT_GLYPH_MAPPING: StrGlyphMapping<'_> = StrGlyphMapping::new(",
        )?;
        write!(file, "    \"")?;
        // group codepoints in ranges of subsequent codepoints
        let mut start = self.glyph_code_points[0];
        let mut last = self.glyph_code_points[0];
        let mut substitute_index = 0;
        for (i, &code_point) in self.glyph_code_points.iter().skip(1).enumerate() {
            // if the code point is '?' then we remember the index
            if code_point == '?' as u16 {
                substitute_index = i + 1;
            }
            if code_point == last + 1 {
                last = code_point;
            } else {
                if start == last {
                    write!(file, "\\u{{{:x}}}", start)?;
                } else {
                    write!(file, "\\0\\u{{{:x}}}\\u{{{:x}}}", start, last)?;
                }
                start = code_point;
                last = code_point;
            }
        }
        if start == last {
            write!(file, "\\u{{{:x}}}", start)?;
        } else {
            write!(file, "\\0\\u{{{:x}}}\\u{{{:x}}}", start, last)?;
        }
        writeln!(file, "\",")?;
        writeln!(file, "    {}", substitute_index)?;
        writeln!(file, ");")?;

        writeln!(
            file,
            "pub const MOGEEFONT_LIGATURE_CODE_POINTS: [&[u16]; {}] = [",
            self.ligature_code_points.len()
        )?;
        for code_points in self.ligature_code_points.iter() {
            write!(file, "    &[")?;
            for (i, code_point) in code_points.iter().enumerate() {
                if i < code_points.len() - 1 {
                    write!(file, "{:#06x}, ", code_point)?;
                } else if i == code_points.len() - 1 {
                    write!(file, "{:#06x}", code_point)?;
                }
            }
            writeln!(file, "],")?;
        }
        writeln!(file, "];")?;
        Ok(())
    }
}

pub struct InputWrapper<T>(T);

impl<P> TryFrom<InputWrapper<P>> for FontData
where
    P: AsRef<Path>,
{
    type Error = std::io::Error;

    fn try_from(font_dir: InputWrapper<P>) -> std::io::Result<Self> {
        // Read the png images from the font directory
        let font_dir = fs::read_dir(font_dir.0)?;

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
            let code_points: Vec<u16> = path
                .file_stem()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .split('_')
                .map(|s| u16::from_str_radix(s, 16).unwrap())
                .collect();

            let code_point = match code_points.len() {
                0 => continue,
                1 => CodePoint::Single(code_points[0]),
                _ => CodePoint::Ligature(code_points),
            };

            all_glyphs.push((code_point, img));
        }

        // First glyphs, then ligatures
        all_glyphs.sort_by(|a, b| a.0.cmp(&b.0));

        let mut left = 0;
        let mut top = 0;
        let mut glyphs_data = Vec::new();

        for (code_point, img) in all_glyphs {
            let width = usize::try_from(img.width()).unwrap();
            if left + width > ATLAS_WIDTH {
                left = 0;
                top += GLYPH_HEIGHT + 1;
            }
            glyphs_data.push(Glyph {
                left,
                top,
                img,
                code_point,
            });
            left += width + 1;
        }

        Ok(FontData::new(glyphs_data, top + GLYPH_HEIGHT))
    }
}
