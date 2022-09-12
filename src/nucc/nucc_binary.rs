use deku::{ctx::Endian, DekuUpdate};
use strum::IntoEnumIterator;

use xfbin_nucc_binary::{
    NuccBinaryParsed, NuccBinaryParsedReader, NuccBinaryParsedWriter, NuccBinaryType,
};

use super::*;

pub struct NuccBinary {
    pub struct_info: NuccStructInfo,
    pub version: u16,

    pub data: Vec<u8>,
}

impl_nucc_info!(NuccBinary, struct_info);

impl NuccBinary {
    pub fn get_binary_type(&self) -> Option<(NuccBinaryType, Endian)> {
        for binary_type in NuccBinaryType::iter() {
            for (pattern, endian) in binary_type.patterns() {
                if pattern.is_match(&self.struct_info.file_path) {
                    return Some((binary_type, endian));
                }
            }
        }

        None
    }

    pub fn parse_data(
        &self,
        binary_type_opt: Option<(NuccBinaryType, Endian)>,
        endianness: Option<Endian>,
        version: usize,
    ) -> Option<Box<dyn NuccBinaryParsed>> {
        binary_type_opt
            .or_else(|| self.get_binary_type())
            .map(|(b_type, endian)| {
                NuccBinaryParsedReader(b_type, &self.data, endianness.unwrap_or(endian), version)
                    .into()
            })
    }

    pub fn update_data(&mut self, nucc_parsed: Box<dyn NuccBinaryParsed>, version: usize) {
        self.data = NuccBinaryParsedWriter(nucc_parsed, version).into();
    }
}

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

        chunk.update().expect("Could not update Binary chunk");

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
