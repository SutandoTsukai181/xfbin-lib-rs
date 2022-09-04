use super::{NuccChunk, NuccChunkType};
use super::super::nucc::NuccStructInfo;

pub struct NuccChunkIndex;

impl NuccChunk for NuccChunkIndex {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkIndex
    }

    fn version(&self) -> u16 {
        0
    }
}

impl NuccChunkIndex {
    pub fn default_chunk_info() -> NuccStructInfo {
        NuccStructInfo {
            chunk_name: String::from("index"),
            file_path: String::from(""),
            chunk_type: NuccChunkType::NuccChunkIndex.to_string(),
        }
    }
}
