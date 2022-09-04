use deku::{ctx, prelude::*};

use super::{NuccChunk, NuccChunkType};
use super::super::nucc::NuccStructInfo;

#[derive(Default)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: ctx::Endian, version: u16",
    ctx_default = "ctx::Endian::Big, 0x79",
)]
pub struct NuccChunkPage {
    #[deku(skip, default = "version")]
    pub version: u16,

    pub map_index_count: u32,
    pub reference_count: u32,
}

impl NuccChunk for NuccChunkPage {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkPage
    }

    fn version(&self) -> u16 {
        self.version
    }
}

impl NuccChunkPage {
    pub fn default_chunk_info() -> NuccStructInfo {
        NuccStructInfo {
            chunk_name: String::from("Page0"),
            file_path: String::from(""),
            chunk_type: NuccChunkType::NuccChunkPage.to_string(),
        }
    }
}
