use std::{rc::Rc, env, fs};

use archives::to_saf_archive;
use block::Block;
use bvpfile::{BVPFile, File, Modality};
use placement::Placement;
use vector3::Vector3;
use xxhash_rust::xxh3::xxh3_64;

mod arguments;
mod block;
mod placement;
mod formats;
mod bvpfile;
mod aux;
mod vector3;
mod archives;


fn volume2block(parent_block_i: usize, dimensions: Vector3<u32>, block_dimensions: Vector3<u32>, format_i: usize, bvp_state: &mut BVPFile) -> Result<(), String> {
    let block_count = (dimensions / block_dimensions).ceil();
    let format = &bvp_state.formats[format_i];
    
    for x in 0..block_count.x {
        for y in 0..block_count.y {
            for z in 0..block_count.z {
                let block_start = block_dimensions * Vector3::from_xyz(x, y, z);
                let block_end = (block_start + block_dimensions).min(&dimensions);

                let block = (&bvp_state.blocks[parent_block_i]).get_data_in_range(block_start, block_end, format)?;
                let block_data = match block.data {
                    Some(d) => d,
                    None => {
                        return Err("Block does not have data".to_string());
                    }
                };
                let block_hash = xxh3_64(&block_data[..]);

                match bvp_state.block_map.get(&block_hash) {
                    Some(block_id) => {
                        bvp_state.blocks[0].placements.push(Placement::new(block_start, block_id.clone()));
                    },
                    None => {
                        let block_id = bvp_state.blocks.len();
                        let block_url = format!("blocks/block_{}.raw", block_id);
                        bvp_state.block_map.insert(block_hash, block_id);
                        let mut new_block = Block::new(block.dimensions, Some(format_i), None);
                        new_block.format = bvp_state.blocks[parent_block_i].format;
                        new_block.data_url = Some(block_url.clone());
                        bvp_state.blocks.push(new_block);
                        bvp_state.files.push(File::new(block_url, Rc::new(block_data), None));
                    }
                };
            }
        }
    }

    return Ok(());
}

fn read_input_file(filepath: &str) -> Result<Vec<u8>, String> {
    match fs::read(filepath) {
        Ok(v) => {
            return Ok(v);
        },
        Err(_) => {
            return Err("Could not read file".to_string());
        }
    }
}

fn main() -> Result<(), String> {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() < 2 {
        return Err("Missing JSON config file".to_string());
    }
    let config_filepath = &arguments[1];
    let parameters = arguments::parse_config(&config_filepath)?;
    let input_data = read_input_file(&parameters.input_file)?;
    let mut bvp_file = BVPFile::new();
    bvp_file.formats.push(parameters.input_format);
    let scale = Vector3 { x: 1.0, y: 1.0, z: 1.0 };
    bvp_file.modalities.push(Modality::new(None, None, None, scale, 0));

    let parent_block = Block::new(parameters.dimensions, Some(0), Some(input_data));
    bvp_file.blocks.push(parent_block);
    
    volume2block(0, parameters.dimensions, parameters.block_dimensions, 0, &mut bvp_file)?;
    
    bvp_file.files.push(File::new("manifest.json".to_string(), Rc::new(bvp_file.to_manifest()?), Some("application/json".to_string())));

    let saf = to_saf_archive(&bvp_file.files)?;
    match fs::write(parameters.output_file, saf) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("Error outputing file: {}", e));
        }
    };

    return Ok(());
}