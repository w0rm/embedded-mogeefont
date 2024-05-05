//! ![Unlike many other pixel fonts, MogeeFont maximizes screen space efficiency by incorporating glyphs of variable width alongside kerning tables and ligatures.](https://raw.githubusercontent.com/w0rm/embedded-mogeefont/main/mogeefont.png)
//!
//! MogeeFont was originally created by Nadya Kuzmina for a pixel game that had to fit on a 64×64 pixel screen. You can read about [the history of MogeeFont here](https://nadyakuzmina.com/story-of-mogeefont.html).
//!
//! This crate brings the font to embedded systems, it should be used together with [embedded-graphics](https://github.com/embedded-graphics/embedded-graphics) and [embedded-text](https://github.com/embedded-graphics/embedded-text).
//!
//! ![Embedded](https://raw.githubusercontent.com/w0rm/embedded-mogeefont/main/embedded.jpg)
//!
//! # Usage
//!
//! ```rust
//! use embedded_text::{style::TextBoxStyle, TextBox};
//! use embedded_mogeefont::MogeeTextStyle;
//! use embedded_graphics::{
//!   geometry::{Size, Point},
//!   mock_display::MockDisplay,
//!   pixelcolor::BinaryColor,
//!   primitives::Rectangle,
//!   Drawable,
//! };
//!
//! let mut display = MockDisplay::new();
//! let character_style = MogeeTextStyle::new(BinaryColor::On);
//! let textbox_style = TextBoxStyle::default();
//! let textbox_bounds = Rectangle::new(Point::zero(), Size::new(42, 22));
//! let textbox = TextBox::with_textbox_style(
//!    "Hello, world!",
//!    textbox_bounds,
//!    character_style,
//!    textbox_style,
//! );
//! textbox.draw(&mut display).unwrap();
//! assert_eq!(
//!   display,
//!   MockDisplay::from_pattern(&[
//!     "                                          ",
//!     "#  #     # #                       #   # #",
//!     "#  #     # #                       #   # #",
//!     "#  #  ## # # ##      # # # ##  ##  #  ## #",
//!     "#### # # # # # #     # # # # # # # # # # #",
//!     "#  # ### # # # #     # # # # # # # # # # #",
//!     "#  # #   # # # #     # # # # # #   # # #  ",
//!     "#  #  ## # #  ## #   ## #   ## #   # ##  #",
//!     "                 #                        ",
//!     "                #                         ",
//!   ]),
//! );
//! ```
//!
//! # Supported characters
//!
//START-SPECIMEN
//! | Charset | Specimen, upscaled to 2x |
//! |---------|----------|
//! | `ASCII` | ![ASCII](data:image/png;base64,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) |
//END-SPECIMEN
//!
#![no_std]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(missing_copy_implementations)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(unstable_features)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
mod charset;
mod draw_target;
mod generated;
mod kerning;
mod ligatures;
mod side_bearings;
mod text_style;

pub use text_style::TextStyle as MogeeTextStyle;
