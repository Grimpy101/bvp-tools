use std::{env, path::Path, fs, str};

use block::Block;
use bvpfile::{File, BVPFile};
use formats::Format;

mod arguments;
mod block;
mod placement;
mod formats;
mod bvpfile;
mod vector3;
mod archives;
mod compression;
mod json_aux;


fn get_files_from_dir(filepath: &Path) -> Result<Vec<File>, String> {
    let vec = Vec::new();
    // TODO: Implement!
    return Ok(vec);
}

fn get_files_from_json(filepath: &Path) -> Result<Vec<File>, String> {
    let vec = Vec::new();
    // TODO: Implement!
    return Ok(vec);
}

fn get_files_from_saf(filepath: &Path) -> Result<Vec<File>, String> {
    let contents = match fs::read(filepath) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("Cannot read file {}: {}", filepath.display(), e));
        }
    };
    let vec = archives::from_saf_archive(&contents)?;
    return Ok(vec);
}

fn find_file<'a>(files: &'a Vec<File>, name: &str) -> Option<&'a File> {
    for file in files {
        if file.name == name {
            return Some(file);
        }
    }
    return None;
}

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

    let bvp_state = BVPFile::from_manifest(content, &files)?;
    return Ok(bvp_state);
}

fn populate_volume(bvp_state: &BVPFile, current_block: usize, new_block: &mut Block, format: &Format) -> Result<(), String> {
    for placement in &bvp_state.blocks[current_block].placements {
        let block_index = placement.block;
        let block = &bvp_state.blocks[block_index];
        if block.data.is_some() {
            let res = new_block.set_data_in_range(placement.position, block, format);
            if res.is_err() {
                return res;
            };
        } else {
            let res = populate_volume(bvp_state, block_index, new_block, format);
            if res.is_err() {
                return res;
            }
        }
    }
    return Ok(());
}

fn find_format(bvp_state: &BVPFile, current_block: usize) -> Result<&Format, String> {
    let mut stack = Vec::new();
    stack.push(current_block);

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
    let input_filepath = Path::new(arguments[1].as_str());
    let files;
    if input_filepath.is_dir() {
        files = get_files_from_dir(input_filepath)?;
    } else if input_filepath.is_file() {
        if input_filepath.extension().is_some() {
            let ext = input_filepath.extension().unwrap();
            if ext == "bvp" || ext == "saf" {
                files = get_files_from_saf(input_filepath)?;
            } else if ext == "json" {
                files = get_files_from_json(input_filepath)?;
            } else {
                return Err(format!("Invalid input: {}", input_filepath.display()));
            }
        } else {
            return Err(format!("Invalid input: {}", input_filepath.display()));
        }
    } else {
        return Err(format!("Invalid input: {}", input_filepath.display()));
    }

    let bvp_state = get_bvp_state(files)?;

    let mut name_index = 0;
    for modality in &bvp_state.modalities {
        let root_block_index = modality.block;
        let root_block = &bvp_state.blocks[root_block_index];
        let format = match find_format(&bvp_state, root_block_index) {
            Ok(format_index) => format_index,
            Err(e) => return Err(e)
        };
        let root_volume_size = format.count_space(root_block.dimensions);
        let mut root_data = Vec::with_capacity(root_volume_size as usize);
        // This expects that the BVP asset is formed correctly
        // and that all blocks in the volume are provided. If a block
        // is somehow left out, the random data will not be overwritten - this is bad!
        unsafe { root_data.set_len(root_volume_size as usize); }
        let mut new_block = Block::new(root_block.dimensions, root_block.format, None);
        new_block.data = Some(root_data);

        let res = populate_volume(&bvp_state, root_block_index, &mut new_block, format);
        if res.is_err() {
            return res;
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
            Err(_) => {
                // TODO: Add some error reporting
                break;
            }
        };

        name_index += 1;
    }

    return Ok(());
}