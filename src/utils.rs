use deku::bitvec;
use deku::ctx;
use deku::prelude::*;

pub fn deku_align(position: u32, align_value: u32) -> u32 {
    match position % align_value {
        0 => 0,
        x => align_value - x,
    }
}

#[derive(Default)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(ctx = "_: ctx::Endian", ctx_default = "ctx::Endian::Big")]
pub struct DekuString {
    #[deku(until = "|v: &u8| *v == 0")]
    data: Vec<u8>,
}

impl From<String> for DekuString {
    fn from(string: String) -> Self {
        DekuString {
            data: (string + "\0").into_bytes(),
        }
    }
}

impl From<DekuString> for String {
    fn from(value: DekuString) -> Self {
        codepage::to_encoding(932)
            .unwrap()
            .decode(&value.data)
            .0
            .trim_end_matches('\0')
            .to_string()
    }
}

pub type DekuBitSlice = bitvec::BitSlice<bitvec::Msb0, u8>;
pub type DekuBitVec = bitvec::BitVec<bitvec::Msb0, u8>;
