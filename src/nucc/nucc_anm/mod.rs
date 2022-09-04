pub mod clump;
pub mod curve;
pub mod entry;

use super::*;

use clump::*;
use curve::*;
use deku::DekuUpdate;
use entry::*;
use hashbrown::{HashMap, HashSet};

pub struct NuccAnm {
    pub struct_info: NuccStructInfo,
    pub version: u16,

    pub clumps: Vec<Clump>,
}

impl_nucc_info!(NuccAnm, struct_info);

impl<'a> From<NuccStructConverter<'a>> for NuccAnm {
    fn from(converter: NuccStructConverter<'a>) -> Self {
        fn process_coord(
            entry_coord: ClumpCoordIndex,
            entry_struct_ref: NuccStructReference,
            processed_entries: &mut HashSet<ClumpCoordIndex>,
            entries_map: &mut HashMap<ClumpCoordIndex, entry::Entry>,
            entry_struct_refs: &Vec<NuccStructReference>,
            clump_parents: &HashMap<ClumpCoordIndex, Vec<ClumpCoordIndex>>,
        ) {
            if processed_entries.contains(&entry_coord) {
                return;
            }

            let mut entry_children = vec![];
            if let Some(children) = clump_parents.get(&entry_coord) {
                entry_children = children
                    .iter()
                    .map(|c| {
                        process_coord(
                            c.clone(),
                            entry_struct_refs[c.1 as usize].clone(),
                            processed_entries,
                            entries_map,
                            entry_struct_refs,
                            clump_parents,
                        );

                        entries_map.remove(c).unwrap()
                    })
                    .collect();
            }

            let entry = entries_map.entry(entry_coord.clone()).or_default();
            entry.entry_info = EntryInfo::StructRef(entry_struct_ref);
            entry.children = entry_children;

            processed_entries.insert(entry_coord.clone());
        }

        fn process_coords(
            clump_index: i16,
            entries_map: &mut HashMap<ClumpCoordIndex, entry::Entry>,
            entry_struct_refs: &Vec<NuccStructReference>,
            clump_parents: &HashMap<ClumpCoordIndex, Vec<ClumpCoordIndex>>,
        ) {
            let mut processed_entries = HashSet::new();

            for (i, entry_struct_ref) in entry_struct_refs.iter().enumerate() {
                let entry_coord = ClumpCoordIndex(clump_index, i as u16);

                process_coord(
                    entry_coord,
                    entry_struct_ref.clone(),
                    &mut processed_entries,
                    entries_map,
                    entry_struct_refs,
                    clump_parents,
                );
            }
        }

        let NuccStructConverter(boxed, struct_infos, struct_references) = converter;
        let chunk = boxed.downcast::<NuccChunkAnm>().map(|c| *c).ok().unwrap();

        // Convert the entries and store them in a HashMap for easy access
        let mut entries_map = HashMap::new();
        for chunk_entry in chunk.entries {
            entries_map
                .try_insert(
                    chunk_entry.coord_index.clone(),
                    entry::Entry::from(chunk_entry),
                )
                .expect("Duplicate animation entries.");
        }

        let mut clump_parents_vec = vec![];
        for _ in 0..chunk.clumps.len() {
            let clump_parents_map: HashMap<ClumpCoordIndex, Vec<ClumpCoordIndex>> = HashMap::new();
            clump_parents_vec.push(clump_parents_map);
        }

        for co in chunk.coord_parents {
            clump_parents_vec[usize::try_from(co.0 .0).expect("Clump index out of range.")]
                .entry(co.0)
                .or_insert(vec![])
                .push(co.1);
        }

        let mut clumps = vec![];
        for (i, clump) in chunk.clumps.iter().enumerate() {
            let struct_ref = struct_references[clump.clump_index as usize].clone();

            let entry_struct_refs: Vec<NuccStructReference> = clump
                .bone_material_indices
                .iter()
                .map(|i| struct_references[*i as usize].clone())
                .collect();
            let model_struct_refs: Vec<NuccStructReference> = clump
                .model_indices
                .iter()
                .map(|i| struct_references[*i as usize].clone())
                .collect();

            let clump_parents = &clump_parents_vec[i];

            process_coords(
                clump.clump_index as i16,
                &mut entries_map,
                &entry_struct_refs,
                clump_parents,
            );

            let clump_keys: Vec<ClumpCoordIndex> = entries_map
                .keys()
                .filter_map(|k| {
                    if k.0 == clump.clump_index as i16 {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();

            let mut root_entries = vec![];
            for k in clump_keys {
                root_entries.push(
                    entries_map
                        .remove(&k)
                        .expect("Could not find a parent entry to add as a root"),
                );
            }

            clumps.push(Clump::new_clump(
                struct_ref,
                root_entries,
                entry_struct_refs,
                model_struct_refs,
            ))
        }

        let other_entry_struct_infos: Vec<NuccStructInfo> = chunk
            .other_entry_chunk_indices
            .iter()
            .map(|i| struct_infos[*i as usize].clone())
            .collect();

        if other_entry_struct_infos.len() != 0 {
            let mut other_entries = vec![];

            for (i, info) in other_entry_struct_infos.iter().enumerate() {
                let mut other_entry = entries_map
                    .remove(&ClumpCoordIndex(-1, i as u16))
                    .expect("Could not find coord index for other entries.");
                other_entry.entry_info = EntryInfo::StructInfo(info.clone());

                other_entries.push(other_entry);
            }

            let other_entries_clump = Clump::new_other(other_entries, other_entry_struct_infos);
            clumps.push(other_entries_clump);
        }

        let unk_entry_infos: Vec<NuccStructInfo> = chunk
            .unk_entry_chunk_indices
            .iter()
            .map(|i| struct_infos[*i as usize].clone())
            .collect();

        if unk_entry_infos.len() != 0 {
            panic!("Found unk_entry infos. Please report this to the developer.")
        }

        Self {
            struct_info: Default::default(),
            version: chunk.version,
            clumps,
        }
    }
}

impl<'a> From<NuccChunkConverter<'a>> for Box<NuccChunkAnm> {
    fn from(converter: NuccChunkConverter) -> Self {
        let NuccChunkConverter(boxed, _, _) = converter;
        let anm = boxed.downcast::<NuccAnm>().map(|s| *s).ok().unwrap();

        let mut chunk = NuccChunkAnm::default();
        chunk.version = anm.version;

        chunk.update().expect("Could not update Anm chunk.");
        Box::new(chunk)
    }
}

impl NuccStruct for NuccAnm {
    fn chunk_type(&self) -> NuccChunkType {
        NuccChunkType::NuccChunkAnm
    }

    fn version(&self) -> u16 {
        self.version
    }
}
