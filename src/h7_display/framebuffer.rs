use {core::mem::MaybeUninit, embedded_graphics_core::prelude::*};

#[derive(Debug)]
#[repr(transparent)]
pub struct FrameBuffer<PIX: PixelColor, const WIDTH: usize, const HEIGHT: usize>
where
    [(); WIDTH * HEIGHT]:,
{
    buffer: [MaybeUninit<PIX>; WIDTH * HEIGHT],
}

impl<PIX: PixelColor, const WIDTH: usize, const HEIGHT: usize> FrameBuffer<PIX, WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT]:,
{
    pub const fn new() -> Self {
        Self {
            buffer: MaybeUninit::<PIX>::uninit_array(),
        }
    }

    #[inline(always)]
    pub fn at(&self, x: usize, y: usize) -> &PIX {
        let idx = Self::xy_to_index(x, y);
        &self[idx]
    }

    #[inline(always)]
    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut PIX {
        let idx = Self::xy_to_index(x, y);
        &mut self[idx]
    }

    #[inline(always)]
    pub fn xy_to_index(x: usize, y: usize) -> usize {
        x + (WIDTH * y)
    }

    // #[inline(always)]
    // pub unsafe fn at_unchecked(&self, x: usize, y: usize) -> &PIX {
    //     let idx = self.xy_to_index(x, y);
    //     self.get_unchecked(idx)
    // }

    // #[inline(always)]
    // pub unsafe fn at_unchecked_mut(&mut self, x: usize, y: usize) -> &mut PIX {
    //     let idx = self.xy_to_index(x, y);
    //     self.get_unchecked_mut(idx)
    // }
}

impl<PIX: PixelColor, const WIDTH: usize, const HEIGHT: usize> core::ops::Deref
    for FrameBuffer<PIX, WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT]:,
{
    type Target = [PIX; WIDTH * HEIGHT];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(&self.buffer) }
    }
}

impl<PIX: PixelColor, const WIDTH: usize, const HEIGHT: usize> core::ops::DerefMut
    for FrameBuffer<PIX, WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT]:,
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::mem::transmute(&mut self.buffer) }
    }
}
