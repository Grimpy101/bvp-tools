use std::{env, path::Path, fs, str};

use block::Block;
use bvpfile::{File, BVPFile};

mod arguments;
mod block;
mod placement;
mod formats;
mod bvpfile;
mod aux;
mod vector3;
mod archives;
mod compression;
mod json_aux;


fn get_files_from_dir(filepath: &Path) -> Result<Vec<File>, String> {
    let vec = Vec::new();

    return Ok(vec);
}

fn get_files_from_json(filepath: &Path) -> Result<Vec<File>, String> {
    let vec = Vec::new();

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
    for modality in bvp_state.modalities {
        let root_block_index = modality.block;
        let root_block = &bvp_state.blocks[root_block_index];
        let format = match &root_block.format {
            Some(format_index) => &bvp_state.formats[*format_index],
            None => {
                match &root_block.placements.first() { // Might be a little messy
                    Some(f) => {
                        let block = &bvp_state.blocks[f.block];
                        if block.format.is_none() {
                            // TODO: Add some error reporting here
                            continue;
                        }
                        &bvp_state.formats[block.format.unwrap()]
                    },
                    _ => {
                        continue;
                    }
                }
            }
        };
        let root_volume_size = format.count_space(root_block.dimensions);
        let mut root_data = Vec::with_capacity(root_volume_size as usize);
        unsafe { root_data.set_len(root_volume_size as usize); }
        let mut new_block = Block::new(root_block.dimensions, root_block.format, None);
        new_block.data = Some(root_data);

        let mut error = false;
        for placement in &bvp_state.blocks[root_block_index].placements {
            let block_index = placement.block;
            let block = &bvp_state.blocks[block_index];
            match new_block.set_data_in_range(placement.position, block, &format) {
                Ok(()) => (),
                Err(_) => {
                    error = true;
                    break;
                }
            };
        }
        
        let volume_name = match modality.name {
            Some(n) => {
                println!("aha");
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

        if error {
            break;
        }
        name_index += 1;
    }

    return Ok(());
}