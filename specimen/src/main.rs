#![no_std]
#![no_main]
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::text::Baseline;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{Point, WebColors},
    text::Text,
    Drawable,
};
use embedded_graphics_web_simulator::{
    display::WebSimulatorDisplay, output_settings::OutputSettingsBuilder,
};
use embedded_mogeefont::TextStyle;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let output_settings = OutputSettingsBuilder::new().scale(4).build();
    let mut text_display = WebSimulatorDisplay::new((128, 64), &output_settings, None);

    let style = TextStyle::new(Rgb565::CSS_WHITE);

    text_display
        .clear(Rgb565::CSS_BLACK)
        .expect("could not clear display");

    Text::with_baseline(
        "Hello,Rust!\nMogeefont!",
        Point::new(12, 16),
        style,
        Baseline::Top,
    )
    .draw(&mut text_display)
    .expect("could not draw text");

    text_display.flush().expect("could not flush buffer");

    Ok(())
}
