use std::{
    collections::VecDeque,
    iter,
    slice::{Iter, IterMut},
    vec::IntoIter,
};

use super::{curve::Curve, Channel, CurveChunkConverter, NuccStructInfo, NuccStructReference};
use crate::nucc_chunk::{Entry as ChunkEntry, EntryFormat as ChunkEntryFormat};

#[derive(Debug)]
pub struct Entry {
    pub entry_info: EntryInfo,
    pub entry_format: EntryFormat,

    pub curves: Vec<Curve>,

    pub children: Vec<Entry>,
}

impl From<ChunkEntry> for Entry {
    fn from(chunk: ChunkEntry) -> Self {
        let entry_format = match chunk.entry_format {
            ChunkEntryFormat::Coord => EntryFormat::Coord,
            ChunkEntryFormat::Camera => EntryFormat::Camera,
            ChunkEntryFormat::Material => EntryFormat::Material,
            ChunkEntryFormat::LightDirc => EntryFormat::LightDirc,
            ChunkEntryFormat::LightPoint => EntryFormat::LightPoint,
            ChunkEntryFormat::Ambient => EntryFormat::Ambient,
        };

        let channels: Vec<Channel> = entry_format.iter_channels().collect();

        let mut curves = vec![];
        for (header, curve_chunk) in chunk
            .curve_headers
            .into_iter()
            .zip(chunk.curves.into_iter())
        {
            curves.push(Curve::from(CurveChunkConverter(
                channels[header.curve_index as usize].clone(),
                header,
                curve_chunk,
            )));
        }

        Self {
            entry_info: EntryInfo::StructInfo(Default::default()),
            entry_format,
            curves,
            children: Default::default(),
        }
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            entry_info: EntryInfo::StructInfo(Default::default()),
            entry_format: EntryFormat::Coord,
            curves: Default::default(),
            children: Default::default(),
        }
    }
}

impl Entry {
    pub fn iter(&self) -> Iter<Entry> {
        self.children.iter()
    }

    pub fn iter_recursive(&self) -> EntryIter {
        EntryIter::from_entries(&self.children[..])
    }

    pub fn iter_mut(&mut self) -> IterMut<Entry> {
        self.children.iter_mut()
    }

    // pub fn iter_mut_recursive(&mut self) -> EntryIterMut {
    //     EntryIterMut::from_entries(&mut self.children[..])
    // }
}

pub struct EntryIter<'a> {
    stack: VecDeque<Iter<'a, Entry>>,
}

impl<'a> EntryIter<'a> {
    pub fn from_entries(entries: &'a [Entry]) -> Self {
        let mut stack = VecDeque::new();
        stack.push_front(entries.iter());

        Self { stack }
    }
}

impl<'a> Iterator for EntryIter<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stack.front_mut() {
            Some(it) => match it.next() {
                Some(item) => {
                    self.stack.push_front(item.children.iter());
                    Some(item)
                }
                None => {
                    self.stack.pop_front();
                    self.next()
                }
            },
            None => None,
        }
    }
}

// pub struct EntryIterMut<'a> {
//     stack: VecDeque<Iter<'a, Entry>>,
// }

// impl<'a> EntryIterMut<'a> {
//     pub fn from_entries(entries: &'a mut [Entry]) -> Self {
//         let mut stack = VecDeque::new();
//         stack.push_front(entries.iter());

//         Self { stack }
//     }
// }

// impl<'a> Iterator for EntryIterMut<'a> {
//     type Item = &'a mut Entry;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self.stack.front_mut() {
//             Some(it) => match it.next() {
//                 Some(item) => {
//                     self.stack.push_front(item.children.iter());
//                     Some(item)
//                 }
//                 None => {
//                     self.stack.pop_front();
//                     self.next()
//                 }
//             },
//             None => None,
//         }
//     }
// }

#[derive(Debug)]
pub enum EntryFormat {
    Coord,
    Camera,
    Material,
    LightDirc,
    LightPoint,
    Ambient,
}

impl EntryFormat {
    pub fn iter_channels(&self) -> IntoIter<Channel> {
        match self {
            EntryFormat::Coord => vec![
                Channel::Location,
                Channel::Rotation,
                Channel::Scale,
                Channel::Opacity,
            ],
            EntryFormat::Camera => vec![Channel::Location, Channel::Rotation, Channel::Fov],
            EntryFormat::Material => iter::repeat(Channel::Property).take(18).collect(),
            EntryFormat::LightDirc => vec![Channel::Color, Channel::Property, Channel::Rotation],
            EntryFormat::LightPoint => vec![
                Channel::Color,
                Channel::Property,
                Channel::Location,
                Channel::Property,
                Channel::Property,
            ],
            EntryFormat::Ambient => vec![Channel::Color, Channel::Property],
        }
        .into_iter()
    }
}

#[derive(Debug)]
pub enum EntryInfo {
    StructInfo(NuccStructInfo),
    StructRef(NuccStructReference),
}
