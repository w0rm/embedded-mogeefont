use crate::{charset::Charset, draw_target::MogeeFontDrawTarget, generated::ASCII};
use embedded_graphics::{
    draw_target::{DrawTarget, DrawTargetExt},
    geometry::{Point, Size},
    image::ImageDrawable,
    pixelcolor::{BinaryColor, PixelColor},
    primitives::{PrimitiveStyle, Rectangle, StyledDrawable},
    text::{
        renderer::{CharacterStyle, TextMetrics, TextRenderer},
        Baseline, DecorationColor,
    },
};

/// Style properties for text using MogeeFont.
///
/// To create a `TextStyle` with a given text color, use the [`TextStyle::new`] method.
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct TextStyle<C> {
    /// Text color.
    text_color: Option<C>,

    /// Background color.
    background_color: Option<C>,

    /// Charset to use.
    charset: &'static Charset,
}

impl<C> TextStyle<C> {
    /// Creates a new text style with the given text color and the default ASCII charset.
    pub fn new(text_color: C) -> Self {
        Self {
            text_color: Some(text_color),
            background_color: None,
            charset: &ASCII,
        }
    }

    /// Draws the text using the binary color format.
    fn draw_string_binary<D>(
        &self,
        text: &str,
        position: Point,
        mut target: D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let mut pos = position;
        let mut prev_glyph = None;

        for glyph in self.charset.glyph_indices(text) {
            pos.x += self.charset.spacing(prev_glyph, glyph);
            let area = self.charset.glyph_area(glyph);
            self.charset
                .image
                .draw_sub_image(&mut target.translated(pos), &area)?;
            pos.x += area.size.width as i32;
            prev_glyph = Some(glyph);
        }

        Ok(pos)
    }

    /// Returns the x position after the last character in the line of text.
    fn advance_position(&self, text: &str, x: i32) -> i32 {
        let mut x = x;
        let mut prev_glyph = None;
        for glyph in self.charset.glyph_indices(text) {
            x += self.charset.spacing(prev_glyph, glyph);
            x += self.charset.glyph_width(glyph);
            prev_glyph = Some(glyph);
        }
        return x;
    }

    /// Returns the vertical offset between the line position and the top edge of the bounding box.
    fn baseline_offset(&self, baseline: Baseline) -> i32 {
        match baseline {
            Baseline::Top => 0,
            Baseline::Bottom => (self.charset.line_height - 1) as i32,
            Baseline::Middle => ((self.charset.line_height - 1) / 2) as i32,
            Baseline::Alphabetic => self.charset.baseline as i32,
        }
    }
}

impl<C> TextRenderer for TextStyle<C>
where
    C: PixelColor,
{
    type Color = C;

    fn draw_string<D>(
        &self,
        text: &str,
        position: Point,
        baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let position = position - Point::new(0, self.baseline_offset(baseline));

        // Avoid measuring the text twice if the background color is not set.
        let mut offset = None;

        // We have to draw the background first, because if a glyph has a negative
        // left side bearing or kerning, it might overlap the previous glyph.
        if let Some(color) = self.background_color {
            let bg_width = self.advance_position(text, 0);
            let bg_style = PrimitiveStyle::with_fill(color);
            Rectangle::new(
                position,
                Size::new(bg_width as u32, self.charset.line_height),
            )
            .draw_styled(&bg_style, target)?;
            offset = Some(bg_width);
        }

        // Draw the text.
        if let Some(color) = self.text_color {
            let pos =
                self.draw_string_binary(text, position, MogeeFontDrawTarget::new(target, color))?;
            offset = Some(pos.x - position.x);
        };

        Ok(position
            + Point::new(
                offset.unwrap_or_else(|| self.advance_position(text, 0)),
                self.baseline_offset(baseline),
            ))
    }

    fn draw_whitespace<D>(
        &self,
        width: u32,
        position: Point,
        baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let position = position - Point::new(0, self.baseline_offset(baseline));
        if let Some(color) = self.background_color {
            let bg_style = PrimitiveStyle::with_fill(color);
            Rectangle::new(position, Size::new(width, self.charset.line_height))
                .draw_styled(&bg_style, target)?;
        }
        Ok(position + Point::new(width as i32, self.baseline_offset(baseline)))
    }

    fn measure_string(&self, text: &str, position: Point, baseline: Baseline) -> TextMetrics {
        // The bounding box position can be to the left of the position,
        // when the first character has a negative left side bearing,
        // e.g. letter 'j'.
        let bb_left = self
            .charset
            .glyph_indices(text)
            .next()
            .map(|c| self.charset.spacing(None, c))
            .unwrap_or_default();

        let bb_position = position + Point::new(bb_left, -self.baseline_offset(baseline));
        let bb_width = self.advance_position(text, -bb_left);
        let bb_size = Size::new(bb_width as u32, self.charset.line_height);

        TextMetrics {
            bounding_box: Rectangle::new(bb_position, bb_size),
            next_position: position + Point::new(bb_width + bb_left, 0),
        }
    }

    fn line_height(&self) -> u32 {
        self.charset.line_height
    }
}

impl<C> CharacterStyle for TextStyle<C>
where
    C: PixelColor,
{
    type Color = C;

    /// Sets the text color.
    fn set_text_color(&mut self, text_color: Option<Self::Color>) {
        self.text_color = text_color;
    }

    /// Sets the background color.
    fn set_background_color(&mut self, background_color: Option<Self::Color>) {
        self.background_color = background_color;
    }

    /// Underline is not supported.
    fn set_underline_color(&mut self, _underline_color: DecorationColor<Self::Color>) {}

    /// Strikethrough is not supported.
    fn set_strikethrough_color(&mut self, _strikethrough_color: DecorationColor<Self::Color>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::{geometry::Size, mock_display::MockDisplay};
    const TEXT_POS: Point = Point::new(4, 6);

    #[test]
    fn test_draw_string() {
        let style = TextStyle::new(BinaryColor::On);
        let mut display = MockDisplay::new();
        let result = style.draw_string("Hello, World!", Point::zero(), Baseline::Top, &mut display);
        assert_eq!(result, Ok(Point::new(45, 0)));
        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                                             ",
                "#  #     # #          #  #  #         #   # #",
                "#  #     # #          #  #  #         #   # #",
                "#  #  ## # # ##       #  #  # ##  ##  #  ## #",
                "#### # # # # # #      #  #  # # # # # # # # #",
                "#  # ### # # # #      #  #  # # # # # # # # #",
                "#  # #   # # # #      # ## #  # # #   # # #  ",
                "#  #  ## # #  ## #     #  #    ## #   # ##  #",
                "                 #                           ",
                "                #                            ",
            ])
        );
    }

    #[test]
    fn measured_position_should_be_same_as_drawn_for_all_baselines_and_styles() {
        let text = "just a test!";
        let default = TextStyle::new(BinaryColor::On);
        let background = {
            let mut s = TextStyle::new(BinaryColor::On);
            s.set_text_color(None);
            s.set_background_color(Some(BinaryColor::On));
            s
        };
        let transparent = {
            let mut s = TextStyle::new(BinaryColor::On);
            s.set_text_color(None);
            s
        };

        let mut display = MockDisplay::new();
        let text_pos = Point::new(2, 15); // bottom aligned text needs more space
        display.set_allow_overdraw(true);
        for baseline in [
            Baseline::Top,
            Baseline::Bottom,
            Baseline::Alphabetic,
            Baseline::Middle,
        ] {
            for style in &[default, transparent, background] {
                let result = style.draw_string(text, text_pos, baseline, &mut display);
                let text_metrics = style.measure_string(text, text_pos, baseline);
                assert_eq!(result, Ok(text_metrics.next_position));
            }
        }
    }

    #[test]
    fn test_draw_string_with_background() {
        let mut style = TextStyle::new(BinaryColor::On);
        style.set_background_color(Some(BinaryColor::Off));
        let mut display = MockDisplay::new();
        display.set_allow_overdraw(true);
        // Offset the text position by 2 because of negative left side bearing of letter 'j'
        let result =
            style.draw_string("just a test", Point::new(2, 1), Baseline::Top, &mut display);
        assert_eq!(result, Ok(Point::new(41, 1)));

        // Background shouldn't be drawn to the left of letter 'j',
        // because it has a negative left side bearing.
        assert_eq!(
            display,
            MockDisplay::from_pattern(&[
                "                                         ",
                "  .......................................",
                "  #......................................",
                "  ..........#.............#...........#..",
                "  #.#.#..##.###....##.....###..##..##.###",
                "  #.#.#.#...#........#....#...#.#.#...#..",
                "  #.#.#..#..#.#....###....#.#.###..#..#.#",
                "  #.#.#...#.#.#....#.#....#.#.#.....#.#.#",
                "  #.###.##...##.....##.....##..##.##...##",
                "  #......................................",
                "  #......................................",
                "##.......................................",
            ])
        );
    }

    #[test]
    fn test_measure_string() {
        let style = TextStyle::new(BinaryColor::On);
        let metrics = style.measure_string("Hello, World!", TEXT_POS, Baseline::Top);
        assert_eq!(metrics.bounding_box.top_left, TEXT_POS);
        assert_eq!(metrics.bounding_box.size, Size::new(45, 11));
        assert_eq!(metrics.next_position, TEXT_POS + Point::new(45, 0));
    }

    #[test]
    fn test_measure_string_negative_left_bearing() {
        let style = TextStyle::new(BinaryColor::On);
        let metrics = style.measure_string("j", TEXT_POS, Baseline::Top);
        assert_eq!(metrics.bounding_box.top_left - TEXT_POS, Point::new(-2, 0));
        assert_eq!(metrics.bounding_box.size, Size::new(3, 11));
        assert_eq!(metrics.next_position - TEXT_POS, Point::new(1, 0));
    }

    #[test]
    fn test_measure_string_baseline_bottom() {
        let style = TextStyle::new(BinaryColor::On);
        let metrics = style.measure_string("Hello, World!", TEXT_POS, Baseline::Bottom);
        assert_eq!(metrics.bounding_box.top_left, Point::new(0, -10) + TEXT_POS);
    }

    #[test]
    fn test_measure_string_baseline_middle() {
        let style = TextStyle::new(BinaryColor::On);
        let metrics = style.measure_string("Hello, World!", TEXT_POS, Baseline::Middle);
        assert_eq!(metrics.bounding_box.top_left, TEXT_POS + Point::new(0, -5));
    }

    #[test]
    fn test_measure_string_baseline_alphabetic() {
        let style = TextStyle::new(BinaryColor::On);
        let metrics = style.measure_string("Hello, World!", TEXT_POS, Baseline::Alphabetic);
        assert_eq!(metrics.bounding_box.top_left, TEXT_POS + Point::new(0, -8));
    }
}
