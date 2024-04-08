use embedded_graphics::{
    draw_target::DrawTarget, geometry::Dimensions, iterator::ContiguousIteratorExt,
    pixelcolor::BinaryColor, primitives::Rectangle, Pixel,
};

pub struct MogeeFontDrawTarget<'a, T, C> {
    parent: &'a mut T,
    color: C,
}

impl<'a, T: DrawTarget, C> MogeeFontDrawTarget<'a, T, C> {
    pub fn new(parent: &'a mut T, color: C) -> Self {
        Self { parent, color }
    }
}

impl<T: DrawTarget> DrawTarget for MogeeFontDrawTarget<'_, T, T::Color> {
    type Color = BinaryColor;
    type Error = T::Error;

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let foreground_color = self.color;

        self.parent.draw_iter(
            colors
                .into_iter()
                .into_pixels(area)
                .filter(|Pixel(_, color)| color.is_on())
                .map(|Pixel(pos, _)| Pixel(pos, foreground_color)),
        )
    }

    fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        unreachable!()
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        match color {
            BinaryColor::On => self.parent.fill_solid(area, self.color),
            BinaryColor::Off => Ok(()),
        }
    }

    fn clear(&mut self, _color: Self::Color) -> Result<(), Self::Error> {
        unreachable!()
    }
}

impl<T: DrawTarget, C> Dimensions for MogeeFontDrawTarget<'_, T, C> {
    fn bounding_box(&self) -> Rectangle {
        self.parent.bounding_box()
    }
}
