use binrw::BinRead;
use image::Rgb;

#[derive(BinRead, Debug, Default, Clone, Copy)]
pub struct ColorRGBExp32 {
    // #[br(map = |val: u8| (val as f32) / 255.)]
    pub r: u8,
    // #[br(map = |val: u8| (val as f32) / 255.)]
    pub g: u8,
    // #[br(map = |val: u8| (val as f32) / 255.)]
    pub b: u8,
    // #[br(map = |val: i8| 2f32.powi(val as i32))]
    pub exponent: i8,
}

impl ColorRGBExp32 {
    pub fn to_rgb32f(&self) -> Rgb<f32> {
        let scale = 2f32.powi(self.exponent as i32);

        [self.r, self.g, self.b].map(|v| scale * v as f32).into()
    }
}

#[derive(BinRead, Debug, Default, Clone, Copy)]
pub struct CompressedLightCube {
    pub color: [ColorRGBExp32; 6],
}
