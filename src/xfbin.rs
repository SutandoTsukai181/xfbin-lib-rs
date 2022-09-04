use deku::DekuUpdate;
use hashbrown::HashMap;
use itertools::Itertools;

use crate::utils::DekuString;

use super::nucc::*;

use super::nucc_chunk::*;
use super::xfbin_file::*;

pub struct Xfbin {
    pub version: u16,
    pub pages: Vec<XfbinPage>,
}

#[derive(Default)]
pub struct XfbinPage {
    pub structs: Vec<Box<dyn NuccStruct>>,
    pub struct_infos: Vec<NuccStructInfo>,
    pub struct_references: Vec<NuccStructReference>,
}

impl XfbinPage {
    pub fn has_unknown_chunk(&self) -> bool {
        for nucc_struct in self.structs.iter() {
            match nucc_struct.chunk_type() {
                NuccChunkType::NuccChunkUnknown => return true,
                _ => (),
            }
        }

        false
    }

    pub fn destructure(
        self,
    ) -> (
        Vec<Box<dyn NuccStruct>>,
        HashMap<NuccStructInfo, u32>,
        HashMap<NuccStructReference, u32>,
    ) {
        let mut struct_infos = HashMap::new();
        let mut struct_references = HashMap::new();

        if self.has_unknown_chunk() {
            struct_infos.extend(
                self.struct_infos
                    .into_iter()
                    .enumerate()
                    .map(|(i, s)| (s, i as u32)),
            );
            struct_references.extend(
                self.struct_references
                    .into_iter()
                    .enumerate()
                    .map(|(i, s)| (s, i as u32)),
            );
        }

        (self.structs, struct_infos, struct_references)
    }
}

impl From<XfbinFile> for Xfbin {
    fn from(xfbin: XfbinFile) -> Self {
        let mut pages = Vec::new();
        let mut page = XfbinPage::default();

        let chunk_names = &Vec::<String>::from(xfbin.index.chunk_names)[..];
        let file_paths = &Vec::<String>::from(xfbin.index.file_paths)[..];
        let chunk_types = &Vec::<String>::from(xfbin.index.chunk_types)[..];

        let mut struct_infos_index: usize = 0;
        let mut struct_references_index: usize = 0;

        let struct_infos = Vec::<NuccStructInfo>::from(XfbinChunkMapConverter {
            maps: xfbin.index.chunk_maps,
            chunk_names,
            file_paths,
            chunk_types,
        });

        let struct_references = Vec::<NuccStructReference>::from(XfbinChunkReferenceConverter(
            xfbin.index.chunk_references,
            chunk_names,
            &struct_infos[..],
        ));

        let struct_infos_mapped = xfbin
            .index
            .chunk_map_indices
            .into_iter()
            .map(|u| struct_infos[u as usize].clone())
            .collect::<Vec<NuccStructInfo>>();

        for chunk in xfbin.chunks {
            let NuccStructInfo {
                chunk_name,
                file_path,
                chunk_type,
            } = &struct_infos_mapped[struct_infos_index + chunk.chunk_map_index as usize];

            let parsed = chunk.unpack(chunk_type);

            match parsed.chunk_type() {
                NuccChunkType::NuccChunkNull => continue,
                NuccChunkType::NuccChunkPage => {
                    let NuccChunkPage {
                        version: _,
                        map_index_count: struct_infos_count,
                        reference_count: struct_references_count,
                    } = parsed.downcast::<NuccChunkPage>().map(|c| *c).ok().unwrap();

                    let struct_infos_count = struct_infos_count as usize;
                    let struct_references_count = struct_references_count as usize;

                    page.struct_infos =
                        struct_infos_mapped[struct_infos_index..(struct_infos_index + struct_infos_count)].to_vec();
                    page.struct_references = struct_references
                        [struct_references_index..(struct_references_index + struct_references_count)]
                        .to_vec();

                    pages.push(page);
                    page = XfbinPage::default();

                    struct_infos_index += struct_infos_count;
                    struct_references_index += struct_references_count;

                    continue;
                }
                _ => (),
            }

            let mut parsed_struct = Box::<dyn NuccStruct>::from(NuccStructConverter(
                parsed,
                &struct_infos_mapped[struct_infos_index..],
                &struct_references[struct_references_index..],
            ));

            let mut struct_info = parsed_struct.struct_info();
            struct_info.chunk_name = chunk_name.clone();
            struct_info.file_path = file_path.clone();
            struct_info.chunk_type = chunk_type.clone();

            page.structs.push(parsed_struct);
        }

        Self {
            version: xfbin.header.version,
            pages,
        }
    }
}

impl From<Xfbin> for XfbinFile {
    fn from(xfbin: Xfbin) -> Self {
        fn repack_struct(
            boxed: Box<dyn NuccChunk>,
            struct_info: NuccStructInfo,
            page_struct_infos: &mut HashMap<NuccStructInfo, u32>,
        ) -> XfbinChunk {
            let struct_info_index = page_struct_infos.len() as u32;
            let chunk_map_index = *page_struct_infos
                .entry(struct_info)
                .or_insert(struct_info_index);

            let mut chunk = XfbinChunk::repack(boxed);
            chunk.chunk_map_index = chunk_map_index;

            chunk
        }

        let mut header = XfbinHeader::default();
        header.version = xfbin.version;

        let mut index = XfbinIndex::default();
        index.version = xfbin.version;

        let mut chunks = vec![];

        let mut struct_infos_map = HashMap::<NuccStructInfo, u32>::new();

        let mut chunk_map_indices = vec![];
        let mut struct_references_vec = vec![];

        let null_chunk = repack_struct(
            Box::new(NuccChunkNull(xfbin.version)),
            NuccChunkNull::default_chunk_info(),
            &mut struct_infos_map,
        );
        chunks.push(null_chunk);

        for page in xfbin.pages {
            let (page_structs, mut page_struct_infos, mut page_struct_references) =
                page.destructure();

            let null_chunk = repack_struct(
                Box::new(NuccChunkNull(xfbin.version)),
                NuccChunkNull::default_chunk_info(),
                &mut page_struct_infos,
            );
            chunks.push(null_chunk);

            for mut nucc_struct in page_structs {
                let struct_info = nucc_struct.struct_info().clone();

                let boxed = Box::<dyn NuccChunk>::from(NuccChunkConverter(
                    nucc_struct,
                    &mut page_struct_infos,
                    &mut page_struct_references,
                ));

                chunks.push(repack_struct(boxed, struct_info, &mut page_struct_infos));
            }

            // Add nuccChunkPage map
            repack_struct(
                Box::new(NuccChunkPage::default()),
                NuccChunkIndex::default_chunk_info(),
                &mut page_struct_infos,
            );

            // Add nuccChunkIndex map
            repack_struct(
                Box::new(NuccChunkIndex),
                NuccChunkIndex::default_chunk_info(),
                &mut page_struct_infos,
            );

            // Create final nuccChunkPage
            let page_chunk = repack_struct(
                Box::new(NuccChunkPage {
                    version: xfbin.version,
                    map_index_count: page_struct_infos.len() as u32,
                    reference_count: page_struct_references.len() as u32,
                }),
                NuccChunkPage::default_chunk_info(),
                &mut page_struct_infos,
            );

            chunks.push(page_chunk);

            for struct_info in page_struct_infos
                .into_iter()
                .sorted_by_key(|(_, v)| *v)
                .map(|(k, _)| k)
            {
                let struct_info_index = struct_infos_map.len() as u32;
                chunk_map_indices.push(
                    *struct_infos_map
                        .entry(struct_info)
                        .or_insert(struct_info_index),
                );
            }

            struct_references_vec.extend(
                page_struct_references
                    .into_iter()
                    .sorted_by_key(|(_, v)| *v)
                    .map(|(k, _)| k),
            );
        }

        let mut chunk_type_map = HashMap::new();
        let mut file_path_map = HashMap::new();
        let mut chunk_name_map = HashMap::new();

        // Chunk references are written before chunk maps, which might affect the chunk names order in the final xfbin
        // Correct order would be to write chunk maps first, update them with chunk references, and then write chunk references.
        let chunk_references = struct_references_vec
            .into_iter()
            .map(|struct_reference| {
                let mut chunk_name_index = chunk_name_map.len() as u32;
                chunk_name_index = *chunk_name_map
                    .entry(struct_reference.0.clone())
                    .or_insert(chunk_name_index);

                let mut struct_info_index = struct_infos_map.len() as u32;
                struct_info_index = *struct_infos_map
                    .entry(struct_reference.1.clone())
                    .or_insert(struct_info_index);

                (chunk_name_index, struct_info_index)
            })
            .collect_vec();

        let chunk_maps = struct_infos_map
            .into_iter()
            .sorted_by_key(|(_, v)| *v)
            .map(|(struct_info, _)| {
                let mut chunk_type_index = chunk_type_map.len() as u32;
                chunk_type_index = *chunk_type_map
                    .entry(struct_info.chunk_type.clone())
                    .or_insert(chunk_type_index);

                let mut file_path_index = file_path_map.len() as u32;
                file_path_index = *file_path_map
                    .entry(struct_info.file_path.clone())
                    .or_insert(file_path_index);

                let mut chunk_name_index = chunk_name_map.len() as u32;
                chunk_name_index = *chunk_name_map
                    .entry(struct_info.chunk_name.clone())
                    .or_insert(chunk_name_index);

                XfbinChunkMap {
                    chunk_type_index,
                    file_path_index,
                    chunk_name_index,
                }
            })
            .collect_vec();

        let chunk_types = XfbinDataBuffer::<DekuString>::from(
            chunk_type_map
                .into_iter()
                .sorted_by_key(|(_, v)| *v)
                .map(|(k, _)| k)
                .collect_vec(),
        );
        let file_paths = XfbinDataBuffer::<DekuString>::from(
            file_path_map
                .into_iter()
                .sorted_by_key(|(_, v)| *v)
                .map(|(k, _)| k)
                .collect_vec(),
        );
        let chunk_names = XfbinDataBuffer::<DekuString>::from(
            chunk_name_map
                .into_iter()
                .sorted_by_key(|(_, v)| *v)
                .map(|(k, _)| k)
                .collect_vec(),
        );

        index.chunk_types = chunk_types;
        index.file_paths = file_paths;
        index.chunk_names = chunk_names;

        index.chunk_maps = chunk_maps;
        index.chunk_references = chunk_references;
        index.chunk_map_indices = chunk_map_indices;

        header.update().expect("Could not update Xfbin header.");
        index.update().expect("Could not update Xfbin index.");

        let mut xfbin_file = Self {
            header,
            index,
            chunks,
        };

        xfbin_file.update().expect("Could not update Xfbin file.");
        xfbin_file
    }
}
