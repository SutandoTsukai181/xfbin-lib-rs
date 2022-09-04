use deku::{ctx, prelude::*};

use crate::utils::*;

use super::curve::*;
use super::ClumpCoordIndex;

#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: ctx::Endian",
    ctx_default = "ctx::Endian::Big"
)]
pub struct Entry {
    pub coord_index: ClumpCoordIndex,
    pub entry_format: EntryFormat,

    #[deku(update = "self.curve_headers.len() as u16")]
    curve_count: u16,

    #[deku(count = "curve_count")]
    pub curve_headers: Vec<CurveHeader>,

    #[deku(reader = "Entry::read_curves(deku::rest, curve_headers)", ctx = "0")]
    pub curves: Vec<Curve>,
}

impl Entry {
    fn read_curves<'a>(
        input: &'a DekuBitSlice,
        curve_headers: &Vec<CurveHeader>,
    ) -> Result<(&'a DekuBitSlice, Vec<Curve>), DekuError> {
        let mut curves = vec![];
        let mut data = input;

        for header in curve_headers {
            let mut curve_size = header.curve_format.size_per_frame() * header.frame_count as usize;

            if curve_size % 4 != 0 {
                curve_size += 4 - (curve_size % 4);
            }

            match Curve::read(data, (ctx::Endian::Big, curve_size)) {
                Ok((rest, value)) => {
                    curves.push(value);
                    data = rest;
                }
                Err(err) => return Err(err),
            }
        }

        Ok((data, curves))
    }
}

#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: ctx::Endian",
    ctx_default = "ctx::Endian::Big",
    type = "u16"
)]
pub enum EntryFormat {
    #[deku(id = "0x01")]
    Coord,
    #[deku(id = "0x02")]
    Camera,
    #[deku(id = "0x04")]
    Material,
    #[deku(id = "0x05")]
    LightDirc,
    #[deku(id = "0x06")]
    LightPoint,
    #[deku(id = "0x08")]
    Ambient,
}
