pub mod error;
pub mod nucc;
mod nucc_chunk;
mod utils;
pub mod xfbin;
mod xfbin_file;

use std::fs;

use deku::{bitvec::BitView, DekuError, DekuRead, DekuWrite};
use utils::*;
use xfbin::*;
use xfbin_file::*;

pub use nucc_chunk::NuccChunkType;

pub fn read_xfbin(file_path: &str) -> Result<Xfbin, DekuError> {
    read_xfbin_bytes(fs::read(file_path).unwrap())
}

pub fn read_xfbin_bytes(bytes: Vec<u8>) -> Result<Xfbin, DekuError> {
    XfbinFile::read(bytes.view_bits(), ()).map(|(_, value)| value.into())
}

pub fn write_xfbin(xfbin: Xfbin, file_path: &str) -> Result<(), DekuError> {
    write_xfbin_bytes(xfbin).map(|output| {
        fs::write(file_path, output).unwrap();
    })
}

pub fn write_xfbin_bytes(xfbin: Xfbin) -> Result<Vec<u8>, DekuError> {
    let mut output = DekuBitVec::new();
    XfbinFile::from(xfbin)
        .write(&mut output, ())
        .map(|_| output.into_vec())
}
