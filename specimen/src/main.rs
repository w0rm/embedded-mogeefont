#![no_std]
#![no_main]
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb565, WebColors},
    prelude::Point,
    primitives::Rectangle,
    Drawable,
};
use embedded_graphics_web_simulator::{
    display::WebSimulatorDisplay, output_settings::OutputSettingsBuilder,
};
use embedded_mogeefont::MogeeTextStyle;
use embedded_text::{style::TextBoxStyle, TextBox};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let output_settings = OutputSettingsBuilder::new().scale(4).build();
    let width = 128;
    let height = 64;
    let mut text_display = WebSimulatorDisplay::new((width, height), &output_settings, None);

    let character_style = MogeeTextStyle::new(Rgb565::CSS_WHITE);
    let textbox_style = TextBoxStyle::default();
    let padding = 2;
    let text_bounds = Rectangle::new(
        Point::new(padding as i32, padding as i32),
        Size::new(width - padding * 2, height - padding * 2),
    );

    // Create the text box and apply styling options.
    let text_box = TextBox::with_textbox_style(
        concat!(
            "Unlike many other pixel fonts, ",
            "MogeeFont maximizes screen space ",
            "efficiency by incorporating glyphs ",
            "of variable width alongside kerning ",
            "tables and ligatures."
        ),
        text_bounds,
        character_style,
        textbox_style,
    );

    text_display
        .clear(Rgb565::CSS_BLACK)
        .expect("could not clear display");

    text_box
        .draw(&mut text_display)
        .expect("could not draw text");

    text_display.flush().expect("could not flush buffer");

    Ok(())
}
