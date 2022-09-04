use super::*;

pub struct NuccUnknown {
    pub struct_info: NuccStructInfo,
    pub version: u16,

    pub data: Vec<u8>,
    pub chunk_type: String,
}

impl_nucc_info!(NuccUnknown, struct_info);

impl<'a> From<NuccStructConverter<'a>> for NuccUnknown {
    fn from(converter: NuccStructConverter<'a>) -> Self {
        let NuccStructConverter(boxed, _, _) = converter;
        let chunk = boxed
            .downcast::<NuccChunkUnknown>()
            .map(|c| *c)
            .ok()
            .unwrap();

        Self {
            struct_info: Default::default(),
            data: chunk.data,
            chunk_type: chunk.chunk_type,
            version: chunk.version,
        }
    }
}

impl<'a> From<NuccChunkConverter<'a>> for Box<NuccChunkUnknown> {
    fn from(converter: NuccChunkConverter) -> Self {
        let NuccChunkConverter(boxed, _, _) = converter;
        let unknown = boxed.downcast::<NuccUnknown>().map(|s| *s).ok().unwrap();

        Box::new(NuccChunkUnknown {
            version: unknown.version,
            data: unknown.data,
            chunk_type: unknown.chunk_type,
        })
    }
}

impl NuccStruct for NuccUnknown {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkUnknown
    }

    fn version(&self) -> u16 {
        self.version
    }
}
