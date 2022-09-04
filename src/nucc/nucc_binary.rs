use deku::DekuUpdate;

use super::*;

pub struct NuccBinary {
    pub struct_info: NuccStructInfo,
    pub version: u16,

    pub data: Vec<u8>,
}

impl_nucc_info!(NuccBinary, struct_info);

impl<'a> From<NuccStructConverter<'a>> for NuccBinary {
    fn from(converter: NuccStructConverter<'a>) -> Self {
        let NuccStructConverter(boxed, _, _) = converter;
        let chunk = boxed
            .downcast::<NuccChunkBinary>()
            .map(|c| *c)
            .ok()
            .unwrap();

        Self {
            struct_info: Default::default(),
            version: chunk.version,
            data: chunk.data,
        }
    }
}

impl<'a> From<NuccChunkConverter<'a>> for Box<NuccChunkBinary> {
    fn from(converter: NuccChunkConverter) -> Self {
        let NuccChunkConverter(boxed, _, _) = converter;
        let binary = boxed.downcast::<NuccBinary>().map(|s| *s).ok().unwrap();

        let mut chunk = NuccChunkBinary {
            version: binary.version,
            size: binary.data.len() as u32,
            data: binary.data,
        };

        chunk.update().expect("Could not update Binary chunk.");

        Box::new(chunk)
    }
}

impl NuccStruct for NuccBinary {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkBinary
    }

    fn version(&self) -> u16 {
        self.version
    }
}
