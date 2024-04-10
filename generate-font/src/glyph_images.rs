use std::path::Path;

pub struct GlyphImages {
    pub code_points_and_images: Vec<(CodePoint, image::GrayImage)>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CodePoint {
    Single(char),
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

impl TryFrom<&Path> for GlyphImages {
    type Error = std::io::Error;

    /// Read the png images from the font directory
    fn try_from(path: &Path) -> std::io::Result<Self> {
        let font_dir = std::fs::read_dir(path)?;
        let mut glyphs: Vec<(CodePoint, image::GrayImage)> = Vec::new();

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

            // We fit dimensions of the image into u8
            if img.width() > 16 || img.height() > 16 {
                panic!("Image dimensions higher than 16: {:?}", path);
            }

            let code_points: Vec<char> = path
                .file_stem()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .split('_')
                .map(|s| char::from_u32(u32::from_str_radix(s, 16).unwrap()).unwrap())
                .collect();

            let code_point = match code_points[..] {
                [] => panic!("No code points found"),
                [code_point] => CodePoint::Single(code_point),
                _ => CodePoint::Ligature(code_points.into_iter().collect()),
            };

            glyphs.push((code_point, img));
        }

        Ok(GlyphImages {
            code_points_and_images: glyphs,
        })
    }
}
