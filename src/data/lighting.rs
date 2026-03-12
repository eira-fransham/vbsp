use binrw::BinRead;

#[derive(BinRead, Debug, Default, Clone, Copy)]
pub struct ColorRGBExp32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub exponent: i8,
}

#[derive(BinRead, Debug, Default, Clone, Copy)]
pub struct CompressedLightCube {
    pub color: [ColorRGBExp32; 6],
}
