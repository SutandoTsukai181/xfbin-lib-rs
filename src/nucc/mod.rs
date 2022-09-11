pub mod nucc_anm;
pub mod nucc_binary;
pub mod nucc_unknown;

use std::fmt;

use downcast_rs::{impl_downcast, Downcast};
use hashbrown::HashMap;

use super::xfbin_file::XfbinChunkMap;

use super::nucc_chunk::*;
pub use nucc_anm::NuccAnm;
pub use nucc_binary::NuccBinary;
pub use nucc_unknown::NuccUnknown;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct NuccStructInfo {
    pub chunk_name: String,
    pub file_path: String,
    pub chunk_type: String,
}

impl fmt::Display for NuccStructInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ Type: \"{}\", Name: \"{}\", Path: \"{}\" }}",
            self.chunk_type, self.chunk_name, self.file_path
        )
    }
}

pub struct XfbinChunkMapConverter<'a> {
    pub maps: Vec<XfbinChunkMap>,
    pub chunk_names: &'a [String],
    pub file_paths: &'a [String],
    pub chunk_types: &'a [String],
}

impl<'a> From<XfbinChunkMapConverter<'a>> for Vec<NuccStructInfo> {
    fn from(converter: XfbinChunkMapConverter) -> Self {
        let XfbinChunkMapConverter {
            maps,
            chunk_names: names,
            file_paths: paths,
            chunk_types: types,
        } = converter;

        maps.into_iter()
            .map(|c| NuccStructInfo {
                chunk_name: names[c.chunk_name_index as usize].clone(),
                file_path: paths[c.file_path_index as usize].clone(),
                chunk_type: types[c.chunk_type_index as usize].clone(),
            })
            .collect()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash)]
pub struct NuccStructReference(pub String, pub NuccStructInfo);

pub struct XfbinChunkReferenceConverter<'a>(
    pub Vec<(u32, u32)>,
    pub &'a [String],
    pub &'a [NuccStructInfo],
);

impl<'a> From<XfbinChunkReferenceConverter<'a>> for Vec<NuccStructReference> {
    fn from(converter: XfbinChunkReferenceConverter<'a>) -> Self {
        let XfbinChunkReferenceConverter(references, names, struct_infos) = converter;

        references
            .into_iter()
            .map(|r| {
                NuccStructReference(
                    names[r.0 as usize].clone(),
                    struct_infos[r.1 as usize].clone(),
                )
            })
            .collect()
    }
}

pub trait NuccInfo {
    fn struct_info(&self) -> &NuccStructInfo;
    fn struct_info_mut(&mut self) -> &mut NuccStructInfo;
}

macro_rules! impl_nucc_info {
    ($struct:ident,$field:ident) => {
        impl NuccInfo for $struct {
            fn struct_info(&self) -> &NuccStructInfo {
                &self.$field
            }

            fn struct_info_mut(&mut self) -> &mut NuccStructInfo {
                &mut self.$field
            }
        }
    };
}

pub(crate) use impl_nucc_info;

pub trait NuccStruct: NuccInfo + Downcast {
    fn chunk_type(&self) -> NuccChunkType;
    fn version(&self) -> u16;
}

impl_downcast!(NuccStruct);

pub struct NuccStructConverter<'a>(
    pub Box<dyn NuccChunk>,
    pub &'a [NuccStructInfo],
    pub &'a [NuccStructReference],
);

pub struct NuccChunkConverter<'a>(
    pub Box<dyn NuccStruct>,
    pub &'a mut HashMap<NuccStructInfo, u32>,
    pub &'a mut HashMap<NuccStructReference, u32>,
);

impl<'a> From<NuccStructConverter<'a>> for Box<dyn NuccStruct> {
    fn from(converter: NuccStructConverter) -> Self {
        match converter.0.chunk_type() {
            NuccChunkType::NuccChunkAnm => Box::new(NuccAnm::from(converter)),
            NuccChunkType::NuccChunkBinary => Box::new(NuccBinary::from(converter)),
            NuccChunkType::NuccChunkUnknown => Box::new(NuccUnknown::from(converter)),
            any => panic!("Unexpected NuccChunkType: {any}"),
        }
    }
}

impl<'a> From<NuccChunkConverter<'a>> for Box<dyn NuccChunk> {
    fn from(converter: NuccChunkConverter) -> Self {
        match converter.0.chunk_type() {
            NuccChunkType::NuccChunkAnm => {
                Box::<NuccChunkAnm>::from(converter) as Box<dyn NuccChunk>
            }
            NuccChunkType::NuccChunkBinary => {
                Box::<NuccChunkBinary>::from(converter) as Box<dyn NuccChunk>
            }
            NuccChunkType::NuccChunkUnknown => {
                Box::<NuccChunkUnknown>::from(converter) as Box<dyn NuccChunk>
            }
            any => panic!("Unexpected NuccChunkType: {any}"),
        }
    }
}
