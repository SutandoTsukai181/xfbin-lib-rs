mod nucc_chunk_anm;
mod nucc_chunk_binary;
mod nucc_chunk_index;
mod nucc_chunk_null;
mod nucc_chunk_page;
mod nucc_chunk_unknown;

use deku::bitvec::{BitView, Msb0};
use deku::prelude::*;
use downcast_rs::{impl_downcast, Downcast};
use std::str::FromStr;
use strum_macros::{Display, EnumString};

use super::utils::*;
pub use nucc_chunk_anm::NuccChunkAnm;
pub use nucc_chunk_binary::NuccChunkBinary;
pub use nucc_chunk_index::NuccChunkIndex;
pub use nucc_chunk_null::NuccChunkNull;
pub use nucc_chunk_page::NuccChunkPage;
pub use nucc_chunk_unknown::NuccChunkUnknown;

pub use nucc_chunk_anm::{ClumpCoordIndex, ParentChildIndex};
pub use nucc_chunk_anm::{Curve, CurveFormat, CurveHeader, Entry, EntryFormat};

pub trait NuccChunk: Downcast {
    fn chunk_type(&self) -> NuccChunkType;
    fn version(&self) -> u16;

    fn read_boxed<'a>(
        input: &'a DekuBitSlice,
        version: u16,
    ) -> Result<(&DekuBitSlice, Box<dyn NuccChunk>), DekuError>
    where
        Self: Sized + DekuRead<'a, (deku::ctx::Endian, u16)>,
    {
        Self::read(input, (deku::ctx::Endian::Big, version))
            .map(|(rest, value)| (rest, Box::new(value) as Box<dyn NuccChunk>))
    }

    fn write_boxed(
        boxed: Box<dyn NuccChunk>,
        output: &mut DekuBitVec,
        version: u16,
    ) -> Result<(), DekuError>
    where
        Self: Sized + DekuWrite<(deku::ctx::Endian, u16)>,
    {
        Self::write(
            &boxed.downcast::<Self>().map(|c| *c).ok().unwrap(),
            output,
            (deku::ctx::Endian::Big, version),
        )
    }
}

impl_downcast!(NuccChunk);

#[derive(Debug, Display, EnumString, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
#[strum(serialize_all = "camelCase")]
pub enum NuccChunkType {
    NuccChunkUnknown, // Not an actual type
    NuccChunkNull,
    NuccChunkPage,
    NuccChunkIndex, // Does not exist as a chunk
    NuccChunkAnm,
    NuccChunkBinary,
}

impl Default for NuccChunkType {
    fn default() -> Self {
        NuccChunkType::NuccChunkUnknown
    }
}

impl NuccChunkType {
    pub fn read_struct<'a>(
        input: &'a DekuBitSlice,
        chunk_type: &str,
        version: u16,
    ) -> Result<(&'a DekuBitSlice, Box<dyn NuccChunk>), DekuError> {
        match NuccChunkType::from_str(chunk_type).unwrap_or_default() {
            NuccChunkType::NuccChunkNull => Ok((input, Box::new(NuccChunkNull(version)))),
            NuccChunkType::NuccChunkPage => NuccChunkPage::read_boxed(input, version),
            NuccChunkType::NuccChunkIndex => Ok((input, Box::new(NuccChunkIndex))),
            NuccChunkType::NuccChunkAnm => NuccChunkAnm::read_boxed(input, version),
            NuccChunkType::NuccChunkBinary => NuccChunkBinary::read_boxed(input, version),
            NuccChunkType::NuccChunkUnknown => Ok((
                input,
                Box::new(NuccChunkUnknown {
                    data: input.to_bitvec().into_vec(),
                    chunk_type: chunk_type.to_string(),
                    version,
                }),
            )),
        }
    }

    pub fn write_struct(boxed: Box<dyn NuccChunk>, version: u16) -> Result<DekuBitVec, DekuError> {
        let mut output = DekuBitVec::new();
        match boxed.chunk_type() {
            NuccChunkType::NuccChunkNull | NuccChunkType::NuccChunkIndex => {
                Ok(()) as Result<(), DekuError>
            }
            NuccChunkType::NuccChunkPage => NuccChunkPage::write_boxed(boxed, &mut output, version),
            NuccChunkType::NuccChunkAnm => NuccChunkAnm::write_boxed(boxed, &mut output, version),
            NuccChunkType::NuccChunkBinary => {
                NuccChunkBinary::write_boxed(boxed, &mut output, version)
            }
            NuccChunkType::NuccChunkUnknown => {
                let mut chunk = boxed
                    .downcast::<NuccChunkUnknown>()
                    .map(|c| *c)
                    .ok()
                    .unwrap();
                output.append(&mut chunk.data.view_bits_mut::<Msb0>().to_bitvec());

                Ok(()) as Result<(), DekuError>
            }
        }
        .map(|_| output)
    }
}
