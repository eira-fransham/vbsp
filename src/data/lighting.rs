use binrw::BinRead;
use image::Pixel;

#[derive(BinRead, Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct ColorRGBExp32 {
    #[br(map = |val: u8| (val as f32) / 255.)]
    pub r: f32,
    #[br(map = |val: u8| (val as f32) / 255.)]
    pub g: f32,
    #[br(map = |val: u8| (val as f32) / 255.)]
    pub b: f32,
    #[br(map = |val: i8| 2f32.powi(val as i32))]
    pub scale: f32,
}

impl Pixel for ColorRGBExp32 {
    type Subpixel = f32;

    const CHANNEL_COUNT: u8 = 4;

    fn channels(&self) -> &[Self::Subpixel] {
        let as_array: &[Self::Subpixel; 4] = unsafe { std::mem::transmute(self) };
        as_array
    }

    fn channels_mut(&mut self) -> &mut [Self::Subpixel] {
        // TODO: Is this valid?
        let as_array: &mut [Self::Subpixel; 4] = unsafe { std::mem::transmute(self) };
        as_array
    }

    const COLOR_MODEL: &'static str = "RGBE";

    fn channels4(
        &self,
    ) -> (
        Self::Subpixel,
        Self::Subpixel,
        Self::Subpixel,
        Self::Subpixel,
    ) {
        (self.r, self.g, self.b, self.scale as _)
    }

    fn from_channels(
        r: Self::Subpixel,
        g: Self::Subpixel,
        b: Self::Subpixel,
        scale: Self::Subpixel,
    ) -> Self {
        Self { r, g, b, scale }
    }

    fn from_slice(slice: &[Self::Subpixel]) -> &Self {
        let as_array: &[Self::Subpixel; 4] = slice.try_into().unwrap();
        unsafe { std::mem::transmute(as_array) }
    }

    fn from_slice_mut(slice: &mut [Self::Subpixel]) -> &mut Self {
        let as_array: &mut [Self::Subpixel; 4] = slice.try_into().unwrap();
        unsafe { std::mem::transmute(as_array) }
    }

    fn to_rgb(&self) -> image::Rgb<Self::Subpixel> {
        [self.r, self.g, self.b].map(|v| v * self.scale).into()
    }

    fn to_rgba(&self) -> image::Rgba<Self::Subpixel> {
        self.to_rgb().to_rgba()
    }

    fn to_luma(&self) -> image::Luma<Self::Subpixel> {
        self.to_rgb().to_luma()
    }

    fn to_luma_alpha(&self) -> image::LumaA<Self::Subpixel> {
        self.to_rgba().to_luma_alpha()
    }

    fn map<F>(&self, mut f: F) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        Self {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
            scale: f(self.scale),
        }
    }

    fn apply<F>(&mut self, mut f: F)
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        self.channels_mut().iter_mut().for_each(|v| {
            *v = f(*v);
        });
    }

    fn map_with_alpha<F, G>(&self, f: F, _: G) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
        G: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        self.map(f)
    }

    fn apply_with_alpha<F, G>(&mut self, f: F, _: G)
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
        G: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        self.apply(f)
    }

    fn map2<F>(&self, other: &Self, mut f: F) -> Self
    where
        F: FnMut(Self::Subpixel, Self::Subpixel) -> Self::Subpixel,
    {
        Self {
            r: f(self.r, other.r),
            g: f(self.g, other.g),
            b: f(self.b, other.b),
            scale: f(self.scale, other.scale),
        }
    }

    fn apply2<F>(&mut self, other: &Self, mut f: F)
    where
        F: FnMut(Self::Subpixel, Self::Subpixel) -> Self::Subpixel,
    {
        self.channels_mut()
            .iter_mut()
            .zip(other.channels())
            .for_each(|(this, other)| {
                *this = f(*this, *other);
            });
    }

    fn invert(&mut self) {
        self.scale *= 1.;
    }

    fn blend(&mut self, other: &Self) {
        let mut rgb = self.to_rgb();
        rgb.blend(&other.to_rgb());
        let [r, g, b] = rgb.0;
        *self = Self { r, g, b, scale: 1. };
    }
}

#[derive(BinRead, Debug, Default, Clone, Copy)]
pub struct CompressedLightCube {
    pub color: [ColorRGBExp32; 6],
}
