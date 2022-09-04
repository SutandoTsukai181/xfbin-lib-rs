use deku::{ctx, prelude::*};

#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: ctx::Endian",
    ctx_default = "ctx::Endian::Big"
)]
pub struct Clump {
    pub clump_index: u32,

    #[deku(update = "self.bone_material_indices.len()")]
    bone_material_count: u16,

    #[deku(update = "self.model_indices.len()")]
    model_count: u16,

    #[deku(count = "bone_material_count")]
    pub bone_material_indices: Vec<u32>,

    #[deku(count = "model_count")]
    pub model_indices: Vec<u32>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: ctx::Endian",
    ctx_default = "ctx::Endian::Big"
)]
pub struct ClumpCoordIndex(pub i16, pub u16);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: ctx::Endian",
    ctx_default = "ctx::Endian::Big"
)]
pub struct ParentChildIndex(pub ClumpCoordIndex, pub ClumpCoordIndex);
