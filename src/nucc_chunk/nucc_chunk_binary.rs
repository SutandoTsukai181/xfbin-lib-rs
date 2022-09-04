use deku::{ctx, prelude::*};

use super::{NuccChunk, NuccChunkType};

#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: ctx::Endian, version: u16",
    ctx_default = "ctx::Endian::Big, 0x79"
)]
pub struct NuccChunkBinary {
    #[deku(skip, default = "version")]
    pub version: u16,

    #[deku(update = "self.data.len()")]
    pub size: u32,

    #[deku(count = "size")]
    pub data: Vec<u8>,
}

impl NuccChunk for NuccChunkBinary {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkBinary
    }

    fn version(&self) -> u16 {
        self.version
    }
}
