use clap::Parser;
use image::GenericImageView;
use std::fs;
use std::process;

// Clapp application parameters
#[derive(Parser)]
struct GenerateFont {
    /// Path to the font files
    #[clap(short, long)]
    font_dir: String,
    /// Path to the output directory
    #[clap(short, long)]
    output_dir: String,
}

fn main() {
    let args = GenerateFont::parse();

    // Read the png images from the font directory
    let font_dir = match fs::read_dir(&args.font_dir) {
        Ok(font) => font,
        Err(e) => {
            eprintln!("Error reading font directory: {}", e);
            process::exit(1);
        }
    };

    // Iterate over the png images in the font directory
    for entry in font_dir {
        let entry = match entry {
            Ok(entry) => match entry.path().extension() {
                Some(ext) if ext != "png" => continue,
                None => continue,
                _ => entry,
            },
            Err(e) => {
                eprintln!("Error reading font directory: {}", e);
                continue;
            }
        };

        // load png image
        let img = match image::open(entry.path()) {
            Ok(img) => img,
            Err(e) => {
                eprintln!("Error reading image: {}", e);
                continue;
            }
        };

        println!("{:?}: {:?}", entry.path(), img.dimensions())
    }
}
