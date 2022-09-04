mod clump;
mod curve;
mod entry;

use deku::{ctx, prelude::*};

use super::{NuccChunk, NuccChunkType};
pub use clump::*;
pub use curve::*;
pub use entry::*;

#[derive(Default)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: ctx::Endian, version: u16",
    ctx_default = "ctx::Endian::Big, 0x79"
)]
pub struct NuccChunkAnm {
    #[deku(skip, default = "version")]
    pub version: u16,

    pub frame_count: u32,
    pub frame_size: u32,

    #[deku(update = "self.entries.len() as u16")]
    entry_count: u16,
    unk: u16,

    #[deku(update = "self.clumps.len() as u16")]
    clump_count: u16,

    #[deku(update = "self.other_entry_chunk_indices.len() as u16")]
    other_entry_count: u16,

    #[deku(update = "self.unk_entry_chunk_indices.len() as u16")]
    unk_entry_count: u16,

    #[deku(update = "self.coord_parents.len() as u16")]
    coord_count: u16,

    #[deku(count = "clump_count")]
    pub clumps: Vec<Clump>,

    #[deku(count = "other_entry_count")]
    pub other_entry_chunk_indices: Vec<u32>,

    #[deku(count = "unk_entry_count")]
    pub unk_entry_chunk_indices: Vec<u32>,

    #[deku(count = "coord_count")]
    pub coord_parents: Vec<ParentChildIndex>,

    #[deku(count = "entry_count")]
    pub entries: Vec<Entry>,
}

impl NuccChunk for NuccChunkAnm {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkAnm
    }

    fn version(&self) -> u16 {
        self.version
    }
}
