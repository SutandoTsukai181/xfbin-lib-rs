use super::{NuccChunk, NuccChunkType};

pub struct NuccChunkUnknown {
    pub data: Vec<u8>,
    pub chunk_type: String,
    pub version: u16,
}

impl NuccChunk for NuccChunkUnknown {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkUnknown
    }

    fn version(&self) -> u16 {
        self.version
    }
}
