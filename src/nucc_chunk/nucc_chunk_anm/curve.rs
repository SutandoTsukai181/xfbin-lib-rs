use deku::{ctx, prelude::*};

#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: ctx::Endian",
    ctx_default = "ctx::Endian::Big"
)]
pub struct CurveHeader {
    pub curve_index: u16,
    pub curve_format: CurveFormat,
    pub frame_count: u16,
    unk_size_flags: u16,
}

#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: ctx::Endian",
    ctx_default = "ctx::Endian::Big",
    type = "u16"
)]
pub enum CurveFormat {
    #[deku(id = "0x05")]
    Vector3Fixed,
    #[deku(id = "0x06")]
    Vector3Linear,
    #[deku(id = "0x07")]
    Vector3Bezier,
    #[deku(id = "0x08")]
    EulerXYZFixed,
    #[deku(id = "0x09")]
    EulerInterpolated,
    #[deku(id = "0x0A")]
    QuatnerionLinear,
    #[deku(id = "0x0B")]
    FloatFixed,
    #[deku(id = "0x0C")]
    FloatLinear,
    #[deku(id = "0x0D")]
    Vector2Fixed,
    #[deku(id = "0x0E")]
    Vector2Linear,
    #[deku(id = "0x0F")]
    OpacityShortTable,
    #[deku(id = "0x10")]
    ScaleShortTable,
    #[deku(id = "0x11")]
    QuaternionShortTable,
    #[deku(id = "0x14")]
    ColorRGBTable,
    #[deku(id = "0x15")]
    Vector3Table,
    #[deku(id = "0x16")]
    FloatTable,
    #[deku(id = "0x17")]
    QuaternionTable,
    #[deku(id = "0x18")]
    FloatTableNoInterp,
    #[deku(id = "0x19")]
    Vector3ShortLinear,
    #[deku(id = "0x1A")]
    Vector3TableNoInterp,
    #[deku(id = "0x1B")]
    QuaternionShortTableNoInterp,
    #[deku(id = "0x1D")]
    OpacityShortTableNoInterp,
}

impl CurveFormat {
    pub fn size_per_frame(&self) -> usize {
        match self {
            CurveFormat::OpacityShortTable | CurveFormat::OpacityShortTableNoInterp => 0x02,
            CurveFormat::ColorRGBTable => 0x03,
            CurveFormat::FloatFixed | CurveFormat::FloatTable | CurveFormat::FloatTableNoInterp => {
                0x04
            }
            CurveFormat::ScaleShortTable => 0x06,
            CurveFormat::FloatLinear
            | CurveFormat::Vector2Fixed
            | CurveFormat::QuaternionShortTable
            | CurveFormat::QuaternionShortTableNoInterp => 0x08,
            CurveFormat::Vector3Fixed
            | CurveFormat::EulerXYZFixed
            | CurveFormat::Vector2Linear
            | CurveFormat::Vector3Table
            | CurveFormat::Vector3TableNoInterp => 0x0C,
            CurveFormat::Vector3Linear | CurveFormat::QuaternionTable => 0x10,
            CurveFormat::QuatnerionLinear => 0x14,
            CurveFormat::Vector3Bezier => todo!(),
            CurveFormat::EulerInterpolated => todo!(),
            CurveFormat::Vector3ShortLinear => todo!(),
        }
    }
}

#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: ctx::Endian, size: usize",
    ctx_default = "ctx::Endian::Big, 0"
)]
pub struct Curve {
    #[deku(count = "size")]
    pub data: Vec<u8>,
}
