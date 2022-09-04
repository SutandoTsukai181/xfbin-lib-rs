use super::super::nucc::NuccStructInfo;
use super::{NuccChunk, NuccChunkType};

pub struct NuccChunkNull(pub u16);

impl NuccChunk for NuccChunkNull {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkNull
    }

    fn version(&self) -> u16 {
        self.0
    }
}

impl NuccChunkNull {
    pub fn default_chunk_info() -> NuccStructInfo {
        NuccStructInfo {
            chunk_name: String::from(""),
            file_path: String::from(""),
            chunk_type: NuccChunkType::NuccChunkNull.to_string(),
        }
    }
}
