use std::ops::Range;

use embedded_graphics_core::prelude::*;
pub use framebuffer::FrameBuffer;

mod framebuffer;

#[derive(Debug)]
pub struct H7Display<'b, COLOR: PixelColor, const WIDTH: usize, const HEIGHT: usize>
where
    [(); WIDTH * HEIGHT]:,
{
    front_buffer_idx: usize,
    buffers: [&'b mut FrameBuffer<COLOR, WIDTH, HEIGHT>; 2],
}

impl<'b, COLOR: PixelColor, const WIDTH: usize, const HEIGHT: usize>
    H7Display<'b, COLOR, WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT]:,
{
    pub const fn new(
        front: &'b mut FrameBuffer<COLOR, WIDTH, HEIGHT>,
        back: &'b mut FrameBuffer<COLOR, WIDTH, HEIGHT>,
    ) -> Self {
        Self {
            front_buffer_idx: 0,
            buffers: [front, back],
        }
    }

    #[inline(always)]
    pub fn front_buffer(&self) -> &FrameBuffer<COLOR, WIDTH, HEIGHT> {
        self.buffers[self.front_buffer_idx]
    }

    #[inline(always)]
    pub fn back_buffer(&self) -> &FrameBuffer<COLOR, WIDTH, HEIGHT> {
        self.buffers[(self.front_buffer_idx + 1) % self.buffers.len()]
    }

    #[inline(always)]
    pub fn back_buffer_mut(&mut self) -> &mut FrameBuffer<COLOR, WIDTH, HEIGHT> {
        unsafe { core::mem::transmute(self.back_buffer()) }
    }

    pub fn swap_buffers(
        &mut self,
    ) -> (
        &FrameBuffer<COLOR, WIDTH, HEIGHT>,
        &mut FrameBuffer<COLOR, WIDTH, HEIGHT>,
    ) {
        self.front_buffer_idx += 1;
        self.front_buffer_idx %= self.buffers.len();

        let front = self.front_buffer();
        let back: &mut FrameBuffer<COLOR, WIDTH, HEIGHT> =
            unsafe { core::mem::transmute(self.back_buffer()) };
        // after swap, copy the new front to the new back
        back.copy_from_slice(&**front);

        (front, back)
    }

    pub fn scroll(&mut self, px: i32, fill: COLOR) {
        match px {
            0 => { /* nop */ }
            i32::MIN..=-1 => {
                for _ in 0..px.abs().min(HEIGHT as i32) {
                    for row in (1..HEIGHT).rev() {
                        let (start, end) = self.back_buffer_mut().split_at_mut(row * WIDTH);
                        // end[..WIDTH].copy_from_slice(&start[Self::row_range(row - 1)]);
                        unsafe {
                            end.get_unchecked_mut(..WIDTH)
                                .copy_from_slice(start.get_unchecked(Self::row_range(row - 1)))
                        };
                    }
                    // self.back_buffer_mut()[Self::row_range(0)].fill(fill);
                    unsafe {
                        self.back_buffer_mut()
                            .get_unchecked_mut(Self::row_range(0))
                            .fill(fill);
                    }
                }
            }
            1..=i32::MAX => {
                for _ in 0..px.min(HEIGHT as i32) {
                    for row in 1..HEIGHT {
                        let (start, end) = self.back_buffer_mut().split_at_mut(row * WIDTH);
                        // start[Self::row_range(row - 1)].copy_from_slice(&end[..WIDTH]);
                        unsafe {
                            start
                                .get_unchecked_mut(Self::row_range(row - 1))
                                .copy_from_slice(end.get_unchecked(..WIDTH));
                        }
                    }
                    // self.back_buffer_mut()[Self::row_range(HEIGHT - 1)].fill(fill);
                    unsafe {
                        self.back_buffer_mut()
                            .get_unchecked_mut(Self::row_range(HEIGHT - 1))
                            .fill(fill)
                    };
                }
            }
        }
    }

    #[inline(always)]
    pub const fn width(&self) -> usize {
        WIDTH
    }

    #[inline(always)]
    pub const fn height(&self) -> usize {
        HEIGHT
    }

    #[inline(always)]
    const fn row_range(row: usize) -> Range<usize> {
        // assert!(row < HEIGHT);
        let start = row * WIDTH;
        start..(start + WIDTH)
    }

    #[inline]
    fn bounded_x(x: i32) -> usize {
        if x < 0 {
            0
        } else if x > WIDTH as i32 {
            WIDTH
        } else {
            x as usize
        }
    }

    #[inline]
    fn bounded_y(y: i32) -> usize {
        if y < 0 {
            0
        } else if y > HEIGHT as i32 {
            HEIGHT
        } else {
            y as usize
        }
    }
}

impl<'b, COLOR: PixelColor, const WIDTH: usize, const HEIGHT: usize> DrawTarget
    for H7Display<'b, COLOR, WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT]:,
{
    type Color = COLOR;
    type Error = std::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x >= 0 && point.x < WIDTH as i32 && point.y >= 0 && point.y < HEIGHT as i32 {
                *self
                    .back_buffer_mut()
                    .at_mut(point.x as usize, point.y as usize) = color;
            }
        }

        Ok(())
    }

    fn fill_contiguous<I>(
        &mut self,
        area: &embedded_graphics_core::primitives::Rectangle,
        colors: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.draw_iter(
            area.points()
                .zip(colors)
                .map(|(pos, color)| embedded_graphics_core::Pixel(pos, color)),
        )
    }

    fn fill_solid(
        &mut self,
        area: &embedded_graphics_core::primitives::Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        // This impl is ~1000x faster than `self.fill_contiguous(area, core::iter::repeat(color))`
        let x_start = Self::bounded_x(area.top_left.x);
        let x_end = Self::bounded_x(area.top_left.x + area.size.width as i32);
        let x_len = x_end - x_start;

        let y_start = Self::bounded_y(area.top_left.y);
        let y_end = Self::bounded_y(area.top_left.y + area.size.height as i32);

        let back_buffer = self.back_buffer_mut();

        for y in y_start..y_end {
            let idx_start = FrameBuffer::<COLOR, WIDTH, HEIGHT>::xy_to_index(x_start, y);
            // back_buffer[idx_start..(idx_start + x_len)].fill(color);
            unsafe {
                back_buffer
                    .get_unchecked_mut(idx_start..(idx_start + x_len))
                    .fill(color);
            };
        }

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_solid(&self.bounding_box(), color)
    }
}

impl<'b, COLOR: PixelColor, const WIDTH: usize, const HEIGHT: usize> OriginDimensions
    for H7Display<'b, COLOR, WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT]:,
{
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}
