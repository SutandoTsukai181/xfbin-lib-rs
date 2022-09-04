use deku::{bitvec::*, ctx::Endian, prelude::*};
use std::{borrow::BorrowMut, marker::PhantomData, mem};

use super::nucc_chunk::{NuccChunk, NuccChunkType};
use super::utils::*;

#[deku_derive(DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct XfbinFile {
    pub header: XfbinHeader,
    pub index: XfbinIndex,

    #[deku(reader = "XfbinFile::read_chunks(deku::rest)")]
    pub chunks: Vec<XfbinChunk>,
}

impl XfbinFile {
    fn read_chunks(input: &DekuBitSlice) -> Result<(&DekuBitSlice, Vec<XfbinChunk>), DekuError> {
        let mut chunks = Vec::new();
        let mut data = input;

        loop {
            match XfbinChunk::read(data, Endian::Big) {
                Ok((rest, value)) => {
                    chunks.push(value);
                    data = rest;
                }
                Err(DekuError::Incomplete(_)) => break,
                Err(err) => return Err(err),
            }
        }

        Ok((data, chunks))
    }
}

#[derive(Default, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian", magic = b"NUCC")]
pub struct XfbinHeader {
    #[deku(
        map = "|v: u32| -> Result<_, DekuError> { Ok(v as u16) }",
        writer = "(self.version as u32).write(deku::output, endian)"
    )]
    pub version: u16,

    #[deku(pad_bytes_before = "1", pad_bytes_after = "6")]
    encrypted: bool,
}

#[derive(Default, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct XfbinIndex {
    #[deku(update = "self.calculate_table_size()")]
    chunk_table_size: u32,
    min_page_size: u32,

    pub version: u16,
    unknown: u16,

    #[deku(update = "(self.chunk_types.count, self.chunk_types.data.len() as u32)")]
    chunk_type_count_size: (u32, u32),

    #[deku(update = "(self.file_paths.count, self.file_paths.data.len() as u32)")]
    file_path_count_size: (u32, u32),

    #[deku(update = "(self.chunk_names.count, self.chunk_names.data.len() as u32)")]
    chunk_name_count_size: (u32, u32),

    #[deku(
        update = "(self.chunk_maps.len() as u32, (mem::size_of::<XfbinChunkMap>() * self.chunk_maps.len()) as u32)"
    )]
    chunk_map_count_size: (u32, u32),

    #[deku(update = "self.chunk_map_indices.len() as u32")]
    chunk_map_index_count: u32,

    #[deku(update = "self.chunk_references.len() as u32")]
    reference_count: u32,

    #[deku(ctx = "*chunk_type_count_size")]
    pub chunk_types: XfbinDataBuffer<DekuString>,

    #[deku(ctx = "*file_path_count_size")]
    pub file_paths: XfbinDataBuffer<DekuString>,

    #[deku(ctx = "*chunk_name_count_size")]
    pub chunk_names: XfbinDataBuffer<DekuString>,

    #[deku(
        count = "(*chunk_map_count_size).0",
        pad_bytes_before = "deku_align(chunk_type_count_size.1 + file_path_count_size.1 + chunk_name_count_size.1, 4)"
    )]
    pub chunk_maps: Vec<XfbinChunkMap>,

    #[deku(count = "reference_count")]
    pub chunk_references: Vec<(u32, u32)>,

    #[deku(count = "chunk_map_index_count")]
    pub chunk_map_indices: Vec<u32>,
}

impl XfbinIndex {
    fn calculate_table_size(&self) -> u32 {
        let string_buffer_size = (self.chunk_types.data.len()
            + self.file_paths.data.len()
            + self.chunk_names.data.len()) as u32;

        0x28 + string_buffer_size
            + deku_align(string_buffer_size, 4)
            + ((mem::size_of::<XfbinChunkMap>() * self.chunk_maps.len())
                + (mem::size_of::<u32>() * self.chunk_map_indices.len())) as u32
    }
}

#[derive(Default)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, (count, size): (u32, u32)"
)]
pub struct XfbinDataBuffer<T> {
    #[deku(count = "size")]
    data: Vec<u8>,

    #[deku(skip, default = "count")]
    count: u32,

    #[deku(skip)]
    phantom: PhantomData<T>,
}

impl From<Vec<String>> for XfbinDataBuffer<DekuString> {
    fn from(strings: Vec<String>) -> Self {
        let mut buf = BitVec::new();

        let count = strings.len() as u32;

        for string in strings {
            DekuString::from(string)
                .write(buf.borrow_mut(), ())
                .unwrap();
        }

        Self {
            data: buf.into_vec(),
            count,
            phantom: PhantomData,
        }
    }
}

impl From<XfbinDataBuffer<DekuString>> for Vec<String> {
    fn from(value: XfbinDataBuffer<DekuString>) -> Self {
        let mut strings = Vec::new();
        let mut data: &DekuBitSlice = value.data.view_bits();

        loop {
            match DekuString::read(data, ()) {
                Ok((rest, value)) => {
                    strings.push(String::from(value));
                    data = rest;
                }
                Err(DekuError::Incomplete(_)) => break,
                Err(_) => panic!("Unable to parse XFBIN strings."),
            }
        }

        if strings.len() as u32 != value.count {
            panic!("Unexpected XFBIN strings count.");
        }

        strings
    }
}

#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "Endian::Big"
)]
pub struct XfbinChunkMap {
    pub chunk_type_index: u32,
    pub file_path_index: u32,
    pub chunk_name_index: u32,
}

#[derive(Default, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct XfbinChunk {
    #[deku(update = "self.chunk_buffer.len()")]
    chunk_size: u32,
    pub chunk_map_index: u32,

    #[deku(pad_bytes_after = "2")]
    pub version: u16,

    #[deku(count = "chunk_size")]
    chunk_buffer: Vec<u8>,
}

impl XfbinChunk {
    pub fn unpack(self, chunk_type: &str) -> Box<dyn NuccChunk> {
        NuccChunkType::read_struct(self.chunk_buffer.view_bits(), chunk_type, self.version)
            .map(|(_, value)| value)
            .unwrap()
    }

    pub fn repack(boxed: Box<dyn NuccChunk>) -> Self {
        let mut value = Self::default();
        value.version = boxed.version();
        value.chunk_buffer = NuccChunkType::write_struct(boxed, value.version)
            .ok()
            .unwrap()
            .into_vec();
        value.update().expect("Could not update Xfbin chunk.");

        value
    }
}
