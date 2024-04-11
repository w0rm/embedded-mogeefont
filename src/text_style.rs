use crate::{
    draw_target::MogeeFontDrawTarget,
    font::{Font, GlyphIndex},
    generated::MOGEEFONT,
};
use az::SaturatingAs;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    image::Image,
    pixelcolor::{BinaryColor, PixelColor},
    primitives::Rectangle,
    text::{
        renderer::{TextMetrics, TextRenderer},
        Baseline,
    },
    Drawable,
};

pub struct TextStyle<'a, C> {
    /// Text color.
    text_color: C,

    /// Font to use.
    font: &'a Font<'a>,
}

impl<'a, C> TextStyle<'a, C> {
    pub fn new(text_color: C) -> Self {
        Self {
            text_color,
            font: &MOGEEFONT,
        }
    }

    fn line_elements<'t>(
        &'a self,
        mut position: Point,
        text: &'t str,
    ) -> impl Iterator<Item = (Point, LineElement)> + 't
    where
        'a: 't,
    {
        let mut chars = self.font.char_offsets(text);
        let mut next_char = chars.next();
        let mut letter_spacing = next_char.map(|c| self.font.letter_spacing(None, c));

        core::iter::from_fn(move || {
            if let Some(spacing) = letter_spacing {
                let p = position;
                position.x += spacing;
                letter_spacing = None;
                Some((p, LineElement::Spacing))
            } else if let Some(c) = next_char {
                let p = position;
                let char_width = self.font.glyph_width(c);
                position.x += char_width;
                next_char = chars.next();
                letter_spacing = next_char.map(|next_c| self.font.letter_spacing(Some(c), next_c));
                Some((p, LineElement::Char(c)))
            } else {
                Some((position, LineElement::Done))
            }
        })
    }

    fn draw_string_binary<D>(
        &self,
        text: &str,
        position: Point,
        mut target: D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        for (p, element) in self.line_elements(position, text) {
            match element {
                LineElement::Char(c) => {
                    let glyph = self.font.glyph(c);
                    Image::new(&glyph, p).draw(&mut target)?;
                }
                LineElement::Spacing => {}
                LineElement::Done => return Ok(p),
            }
        }
        Ok(position)
    }

    /// Returns the position after the last character in the text.
    fn advance_position(&self, text: &str, position: Point) -> Point {
        for (p, element) in self.line_elements(position, text) {
            if let LineElement::Done = element {
                return p;
            }
        }
        position
    }

    /// Returns the vertical offset between the line position and the top edge of the bounding box.
    fn baseline_offset(&self, baseline: Baseline) -> i32 {
        match baseline {
            Baseline::Top => 0,
            Baseline::Bottom => self.font.em_height.saturating_sub(1).saturating_as(),
            Baseline::Middle => (self.font.em_height.saturating_sub(1) / 2).saturating_as(),
            Baseline::Alphabetic => self.font.baseline.saturating_as(),
        }
    }
}

impl<'a, C> TextRenderer for TextStyle<'a, C>
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

        let next = self.draw_string_binary(
            text,
            position,
            MogeeFontDrawTarget::new(target, self.text_color),
        )?;

        Ok(next + Point::new(0, self.baseline_offset(baseline)))
    }

    fn draw_whitespace<D>(
        &self,
        width: u32,
        position: Point,
        _baseline: Baseline,
        _target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        Ok(position + Point::new(width.saturating_as(), 0))
    }

    fn measure_string(&self, text: &str, position: Point, baseline: Baseline) -> TextMetrics {
        // the bounding box position can be to the left of the position,
        // when the first character has a negative left side bearing
        // e.g. letter 'j'
        let bb_position = self
            .line_elements(position, text)
            // skip the first element which is the spacing before the first character
            .nth(1)
            .map(|(p, _)| p)
            .unwrap_or(position)
            - Point::new(0, self.baseline_offset(baseline));

        let bb_width = self.advance_position(text, position).x - position.x;
        let bb_height = self.font.em_height;
        let bb_size = Size::new(bb_width.saturating_as(), bb_height);

        TextMetrics {
            bounding_box: Rectangle::new(bb_position, bb_size),
            next_position: position + bb_size.x_axis(),
        }
    }

    fn line_height(&self) -> u32 {
        self.font.em_height
    }
}

enum LineElement {
    Char(GlyphIndex),
    Spacing,
    Done,
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
        let metrics = style.measure_string("just testing", TEXT_POS, Baseline::Top);
        assert_eq!(metrics.bounding_box.top_left, TEXT_POS + Point::new(-2, 0));
        assert_eq!(metrics.bounding_box.size, Size::new(42, 11));
        assert_eq!(metrics.next_position, TEXT_POS + Point::new(42, 0));
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
