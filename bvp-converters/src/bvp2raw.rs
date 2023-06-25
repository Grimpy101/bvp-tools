use std::{env, path::Path, fs, str};

use bvp::block::Block;
use bvp::bvpfile::BVPFile;
use bvp::file::File;
use bvp::formats::Format;
use bvp::archives::ArchiveEnum;

static HELP: &str = "bvp2raw\n------------\n Usage: bvp2raw <input_file> [<archive type>]\n This message can be viewed with flag `--help`.";

/// Finds a file with given name.
/// * `files` - a list of files
/// * `name` - the name of the file to find
fn find_file<'a>(files: &'a Vec<File>, name: &str) -> Option<&'a File> {
    for file in files {
        if file.name.ends_with(name) {
            return Some(file);
        }
    }
    return None;
}

/// Finds a manifest file and creates BVPFile instance from it.
/// * `files` - a list of files.
fn get_bvp_state(files: Vec<File>) -> Result<BVPFile, String> {
    let manifest = find_file(&files, "manifest.json");
    if manifest.is_none() {
        return Err("Missing manifest file".to_string());
    }
    let manifest = manifest.unwrap();

    let content = match str::from_utf8(&manifest.data) {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("Cannot decode manifest file: {}", e));
        },
    };

    let bvp_state = BVPFile::from_manifest(content, &files).map_err(|x| format!("{}", x))?;
    return Ok(bvp_state);
}

/// Recursively goes through all placements and corresponding blocks,
/// and populates destination block with data from them. Depth first.
/// * `bvp_state` - BVP file state tracker
/// * `current_block_index` - index of the current block (node) being traversed
/// * `dest_block` - destination block
/// * `format` - the format of the data
fn populate_volume(bvp_state: &BVPFile, current_block_index: usize, dest_block: &mut Block, format: &Format) -> Result<(), String> {
    for placement in &bvp_state.blocks[current_block_index].placements {
        let block_index = placement.block;
        let block = &bvp_state.blocks[block_index];
        if block.data.is_some() {
            let res = dest_block.set_data_in_range(placement.position, block, format);
            if res.is_err() {
                return res.map_err(|x| format!("{}", x));
            };
        } else {
            let res = populate_volume(bvp_state, block_index, dest_block, format);
            if res.is_err() {
                return res;
            }
        }
    }
    return Ok(());
}

/// Goes through all nodes in the tree of blocks
/// and finds the first instance of format on a block.
/// It is assumed that formats do not differ inside blocks
/// of the same modality.
/// * `bvp_state` - BVP file state tracker
/// * `current_block`
fn find_format(bvp_state: &BVPFile, current_block_index: usize) -> Result<&Format, String> {
    let mut stack = Vec::new();
    stack.push(current_block_index);

    while !stack.is_empty() {
        let block_index = stack.pop().unwrap();
        let block = &bvp_state.blocks[block_index];
        if block.format.is_some() {
            let format_index = block.format.unwrap();
            return Ok(&bvp_state.formats[format_index]);
        } else {
            for placement in &block.placements {
                let new_block_index = placement.block;
                stack.push(new_block_index);
            }
        }
    }
    return Err("No format found".to_string());
}


fn main() -> Result<(), String> {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() < 2 {
        return Err("Missing input file".to_string());
    }
    
    for arg in &arguments {
        if arg == "--help" {
            println!("{}", HELP);
            return Ok(());
        }
    }

    let input_filepath = Path::new(arguments[1].as_str());
    let archive_tp = if arguments.len() > 2 {
        ArchiveEnum::from_string(arguments[2].clone()).map_err(|x| format!("{}", x))?
    } else {
        ArchiveEnum::None
    };
    let files = archive_tp.read_archive(input_filepath).map_err(|x| format!("{}", x))?;

    let bvp_state = get_bvp_state(files)?;

    let mut name_index = 0;
    let mut errors = Vec::new();
    for modality in &bvp_state.modalities {
        let root_block_index = modality.block;
        let root_block = &bvp_state.blocks[root_block_index];
        let format = match find_format(&bvp_state, root_block_index) {
            Ok(format_index) => format_index,
            Err(e) => {
                errors.push(e.to_string());
                continue;
            }
        };
        let root_volume_size = format.count_space(root_block.dimensions);
        let mut root_data = Vec::with_capacity(root_volume_size as usize);
        // This expects that the BVP asset is formed correctly
        // and that all blocks in the volume are provided. If a block
        // is somehow left out, the random data will not be overwritten - this is bad!
        unsafe { root_data.set_len(root_volume_size as usize); }
        let mut new_block = Block::new(0, root_block.dimensions, root_block.format, None);
        new_block.data = Some(root_data);

        let res = populate_volume(&bvp_state, root_block_index, &mut new_block, format);
        if res.is_err() {
            errors.push(res.unwrap_err().to_string());
            continue;
        }
        
        let volume_name = match modality.name.clone() {
            Some(n) => {
                format!("{}.raw", n)
            },
            None => {
                let filename = input_filepath.file_stem();
                match filename {
                    Some(f) => {
                        format!("{}_volume_{}.raw", f.to_string_lossy(), name_index)
                    },
                    None => {
                        format!("default_volume_{}.raw", name_index)
                    }
                }
            }
        };

        match fs::write(volume_name, new_block.data.unwrap()) {
            Ok(()) => (),
            Err(e) => {
                errors.push(e.to_string());
                continue;
            }
        };

        name_index += 1;
    }

    if errors.len() > 0 {
        let mut message = "Finished with the following errors: ".to_string();

        for error in errors {
            message = format!("{}\n{}", message, error);
        }

        return Err(message);
    }

    return Ok(());
}