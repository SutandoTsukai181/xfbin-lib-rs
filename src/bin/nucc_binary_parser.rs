use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use deku::ctx::Endian;
use itertools::Itertools;
use strum::IntoEnumIterator;
use xfbin_lib_rs::{
    nucc::{nucc_binary::NuccBinary, NuccStructInfo},
    read_xfbin, write_xfbin, NuccChunkType,
};
use xfbin_nucc_binary::{NuccBinaryParsedDeserializer, NuccBinaryParsedSerializer, NuccBinaryType};

use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[clap(name = "nucc_binary_parser")]
#[clap(author = "SutandoTsukai181")]
#[clap(version = "0.1.0")]
#[clap(about = "Unpacks/repacks nuccChunkBinary chunks from XFBIN files into a more usable format", long_about = None)]
struct Args {
    /// Operation mode.
    #[clap(arg_enum, value_parser)]
    mode: Mode,

    /// Path to XFBIN file.
    #[clap(value_parser, value_name = "FILE")]
    xfbin: PathBuf,

    /// Path to binary file. Default is xfbin path with the new file extension.
    #[clap(short, long, value_parser, value_name = "FILE")]
    binary: Option<PathBuf>,

    /// Selected version (index) for reading the binary format. If not given, will be asked for if needed.
    #[clap(short, long, value_parser)]
    selected_version: Option<usize>,

    /// Endianness for reading the binary.
    #[clap(arg_enum, value_parser, default_value = "auto")]
    endian: Endianness,

    /// Overwrite files without asking.
    #[clap(short, long, action)]
    overwrite: bool,

    /// Unpack binary to JSON regardless of the optimal format for the binary file.
    #[clap(short, long, action)]
    json: bool,

    #[clap(subcommand)]
    commands: Option<Commands>,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Mode {
    /// Unpack all supported nuccBinary chunks from the XFBIN.
    Unpack,

    /// Repack a binary chunk into the XFBIN.
    Repack,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Endianness {
    /// Automatically select endianness based on chunk path.
    Auto,

    /// Little endian.
    Little,

    /// Big endian.
    Big,
}

#[derive(Subcommand)]
enum Commands {
    /// List all supported binary types.
    List {
        /// List supported versions for each binary type.
        #[clap(short, long, action)]
        versions: bool,
        /// List some examples of supported chunk file paths for each binary type.
        #[clap(short, long, action)]
        paths: bool,
    },
}

#[derive(Serialize, Deserialize)]
struct MetaData {
    page_index: usize,
    binary_type: String,
    binary_file_name: String,
    struct_info: NuccStructInfo,
}

fn binary_path_from_xfbin(
    xfbin_path: &Path,
    binary_path: Option<PathBuf>,
    mut extension: String,
) -> PathBuf {
    let mut output_path: PathBuf;
    if let Some(ref binary_path) = binary_path {
        output_path = binary_path.clone();
        extension = output_path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap()
            .to_string();
    } else {
        output_path = xfbin_path.to_path_buf();
    }

    output_path.set_file_name(
        output_path
            .file_stem()
            .unwrap_or(xfbin_path.as_os_str())
            .to_str()
            .unwrap()
            .to_string()
            + &extension,
    );

    output_path
}

fn unpack(args: Args) {
    println!(
        "Reading XFBIN: \"{}\"...",
        args.xfbin.file_name().unwrap().to_str().unwrap()
    );

    let xfbin = read_xfbin(&args.xfbin).expect("Could not read XFBIN");

    let endianness = match args.endian {
        Endianness::Auto => None,
        Endianness::Little => Some(Endian::Little),
        Endianness::Big => Some(Endian::Big),
    };

    let mut counter = 0;
    for (page_index, page) in xfbin.pages.iter().enumerate() {
        for nucc_struct in page.structs.iter() {
            if let NuccChunkType::NuccChunkBinary = nucc_struct.chunk_type() {
                let binary = nucc_struct.downcast_ref::<NuccBinary>().unwrap();

                if let Some((binary_type, endian)) = binary.get_binary_type() {
                    println!("Found NuccBinaryType: {}", binary_type);

                    let version = if let Some(selected_version) = args.selected_version {
                        selected_version
                    } else {
                        let versions = binary_type.version_options();
                        if versions.is_empty() {
                            0
                        } else {
                            println!();
                            println!("Select version:");

                            dialoguer::Select::new()
                                .items(&binary_type.version_options())
                                .default(0)
                                .clear(false)
                                .interact()
                                .unwrap_or(0)
                        }
                    };

                    let parsed = binary
                        .parse_data(Some((binary_type, endian)), endianness, version)
                        .unwrap();

                    let mut extension = parsed.extension(args.json);
                    extension = if counter != 0 {
                        format!(".{}", counter)
                    } else {
                        String::from("")
                    } + &extension;

                    let output_path =
                        binary_path_from_xfbin(&args.xfbin, args.binary.clone(), extension);

                    println!(
                        "Writing binary file: \"{}\"...",
                        output_path.file_name().unwrap().to_str().unwrap()
                    );
                    if !args.overwrite
                        && output_path.is_file()
                        && !dialoguer::Confirm::new()
                            .with_prompt("File already exists. Overwrite?")
                            .interact()
                            .unwrap_or(false)
                    {
                        println!("Skipping file.");
                        println!();
                        continue;
                    }

                    fs::write(
                        output_path.clone(),
                        Vec::<u8>::from(NuccBinaryParsedSerializer(parsed, args.json)),
                    )
                    .expect("Could not write binary file");

                    let meta_data = MetaData {
                        page_index,
                        binary_type: binary_type.to_string(),
                        binary_file_name: output_path
                            .clone()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string(),
                        struct_info: binary.struct_info.clone(),
                    };

                    let meta_data = serde_json::to_string_pretty(&meta_data).unwrap();

                    let mut meta_path = output_path.clone();
                    meta_path.set_extension("meta.json");

                    // Don't prompt for overwrite here
                    println!(
                        "Writing metadata: \"{}\"...",
                        meta_path.file_name().unwrap().to_str().unwrap()
                    );
                    fs::write(meta_path, meta_data).expect("Could not write output metadata file");

                    counter += 1;
                } else {
                    println!("Skipping unsupported NuccBinary: {}", binary.struct_info);
                }
            }
        }
    }

    println!();
    println!("Unpacking done.");
}

fn repack(args: Args) {
    println!(
        "Reading XFBIN: \"{}\"...",
        args.xfbin.file_name().unwrap().to_str().unwrap()
    );
    let mut xfbin = read_xfbin(&args.xfbin).expect("Could not read XFBIN");

    let mut meta_path = if let Some(binary_path) = args.binary {
        binary_path
    } else {
        args.xfbin.clone()
    };

    meta_path.set_extension("meta.json");

    println!(
        "Reading metadata: \"{}\"...",
        meta_path.file_name().unwrap().to_str().unwrap()
    );
    let meta_data = fs::read(meta_path.clone()).expect("Could not read metadata file");
    let meta_data: MetaData =
        serde_json::from_slice(&meta_data).expect("Could not parse metadata file");

    let binary_type =
        NuccBinaryType::from_str(&meta_data.binary_type).expect("Unexpected NuccBinaryType");
    println!("Found NuccBinaryType: {}", binary_type);

    let page = xfbin
        .pages
        .get_mut(meta_data.page_index)
        .unwrap_or_else(|| panic!("Could not find page"));

    let nucc_struct = page
        .structs
        .iter_mut()
        .filter(|s| *s.struct_info() == meta_data.struct_info)
        .exactly_one()
        .map_err(|_| {})
        .expect("Could not find a unique nucc struct");

    let mut binary_path = meta_path.clone();
    binary_path.set_file_name(meta_data.binary_file_name);

    println!(
        "Reading binary file: \"{}\"...",
        binary_path.file_name().unwrap().to_str().unwrap()
    );
    let binary_data = fs::read(binary_path).expect("Could not read binary file");
    let converter = NuccBinaryParsedDeserializer(binary_type, args.json, binary_data);

    let binary = nucc_struct.downcast_mut::<NuccBinary>().unwrap();
    binary.update_data(converter.into(), args.selected_version.unwrap_or_default());

    println!(
        "Writing XFBIN: \"{}\"...",
        args.xfbin.file_name().unwrap().to_str().unwrap()
    );
    if !args.overwrite
        && args.xfbin.is_file()
        && !dialoguer::Confirm::new()
            .with_prompt("File already exists. Overwrite?")
            .interact()
            .unwrap_or(false)
    {
        println!("Aborting.");
        println!();
        return;
    }

    write_xfbin(xfbin, &args.xfbin).expect("Could not write XFBIN");

    println!();
    println!("Repacking done.");
}

fn list(print_versions: bool, print_paths: bool) {
    println!("Supported binary types:");

    for binary_type in NuccBinaryType::iter() {
        println!("  {}", binary_type);

        let versions = binary_type.version_options();
        if print_versions {
            if !versions.is_empty() {
                println!("  versions:");
                for (i, version) in versions.iter().enumerate() {
                    println!("      {} - {}", i, version);
                }
            } else {
                println!("  versions: none");
            }

            println!();
        }

        if print_paths {
            println!("  examples:");
            for file_path in binary_type.examples() {
                println!("      {}", file_path);
            }

            println!();
        }
    }

    println!();
}

fn main() {
    let args = Args::parse();

    // Print header
    print!("{}", Args::command().render_version());
    println!("{}", Args::command().get_author().unwrap());
    println!();

    if let Some(commands) = args.commands {
        match commands {
            Commands::List { versions, paths } => {
                list(versions, paths);
            }
        }
    } else {
        match args.mode {
            Mode::Unpack => unpack(args),
            Mode::Repack => repack(args),
        };
    }

    println!("Program finished.");
}
