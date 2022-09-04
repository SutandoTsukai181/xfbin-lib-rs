use std::slice::{Iter, IterMut};

use super::{entry::Entry, EntryIter, NuccStructInfo, NuccStructReference};

#[allow(dead_code)]
pub struct Clump {
    pub clump_info: ClumpInfo,

    pub root_entries: Vec<Entry>,

    // Not sure if there are struct infos that don't have corresponding entries, which is why this is stored here
    entry_struct_refs: Vec<NuccStructReference>,
    model_struct_refs: Vec<NuccStructReference>,

    other_entry_struct_infos: Vec<NuccStructInfo>,
}

impl Clump {
    pub fn new_clump(
        struct_ref: NuccStructReference,
        root_entries: Vec<Entry>,
        entry_struct_refs: Vec<NuccStructReference>,
        model_struct_refs: Vec<NuccStructReference>,
    ) -> Self {
        Self {
            clump_info: ClumpInfo::StructRef(struct_ref),
            root_entries,
            entry_struct_refs,
            model_struct_refs,
            other_entry_struct_infos: Default::default(),
        }
    }

    pub fn new_other(
        other_entries: Vec<Entry>,
        other_entry_struct_infos: Vec<NuccStructInfo>,
    ) -> Self {
        Self {
            clump_info: ClumpInfo::NoInfo,
            root_entries: other_entries,
            entry_struct_refs: Default::default(),
            model_struct_refs: Default::default(),
            other_entry_struct_infos,
        }
    }

    pub fn iter(&self) -> Iter<Entry> {
        self.root_entries.iter()
    }

    pub fn iter_recursive(&self) -> EntryIter {
        EntryIter::from_entries(&self.root_entries[..])
    }

    pub fn iter_mut(&mut self) -> IterMut<Entry> {
        self.root_entries.iter_mut()
    }

    // pub fn iter_mut_recursive(&mut self) -> EntryIterMut {
    //     EntryIterMut::from_entries(&mut self.root_entries[..])
    // }
}

pub enum ClumpInfo {
    NoInfo,
    StructRef(NuccStructReference),
}
