use crate::Font;
use crate::{draw_target::MogeeFontDrawTarget, generated::MOGEEFONT};
use az::SaturatingAs;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    image::Image,
    pixelcolor::{BinaryColor, PixelColor},
    text::{
        renderer::{TextMetrics, TextRenderer},
        Baseline,
    },
    Drawable,
};

pub struct TextStyle<'a, C> {
    /// Text color.
    pub text_color: Option<C>,

    /// Font to use.
    pub font: &'a Font<'a>,
}

impl<'a, C> TextStyle<'a, C> {
    pub fn new(text_color: C) -> Self {
        Self {
            text_color: Some(text_color),
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
        let mut chars = text.chars();
        let mut next_char = chars.next();
        let mut add_spacing = false;

        core::iter::from_fn(move || {
            if add_spacing {
                let p = position;
                position.x += self.font.character_spacing;
                add_spacing = false;
                Some((p, LineElement::Spacing))
            } else if let Some(c) = next_char {
                let p = position;
                let char_width = self.font.glyph_width(c);
                position.x += char_width;
                next_char = chars.next();
                add_spacing = next_char.is_some();
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
            Baseline::Bottom => self.font.character_height.saturating_sub(1).saturating_as(),
            Baseline::Middle => (self.font.character_height.saturating_sub(1) / 2).saturating_as(),
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

        let next = match self.text_color {
            Some(text_color) => self.draw_string_binary(
                text,
                position,
                MogeeFontDrawTarget::new(target, text_color),
            )?,
            None => self.advance_position(text, position),
        };

        Ok(next + Point::new(0, self.baseline_offset(baseline)))
    }

    fn draw_whitespace<D>(
        &self,
        width: u32,
        position: Point,
        baseline: Baseline,
        _target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let position = position - Point::new(0, self.baseline_offset(baseline));
        Ok(position + Point::new(width.saturating_as(), self.baseline_offset(baseline)))
    }

    fn measure_string(&self, text: &str, position: Point, baseline: Baseline) -> TextMetrics {
        let bb_position = position - Point::new(0, self.baseline_offset(baseline));

        let bb_width = self.advance_position(text, position).x - position.x;
        let bb_height = self.font.character_height;

        let bb_size = Size::new(bb_width.saturating_as(), bb_height);

        TextMetrics {
            bounding_box: Rectangle::new(bb_position, bb_size),
            next_position: position + bb_size.x_axis(),
        }
    }

    fn line_height(&self) -> u32 {
        self.font.character_height
    }
}

enum LineElement {
    Char(char),
    Spacing,
    Done,
}
