use std::{rc::Rc, env, fs};

use archives::to_saf_archive;
use block::Block;
use bvpfile::{BVPFile, File, Modality};
use compression::CompressionType;
use placement::Placement;
use vector3::Vector3;
use xxhash_rust::xxh3::xxh3_64;

mod arguments;
mod block;
mod placement;
mod formats;
mod bvpfile;
mod vector3;
mod archives;
mod compression;
mod json_aux;


/// Iterates through blocks of data in volume,
/// schedules unique data for writing to file,
/// and reference data through placements
/// 
/// * `parent_block_i` is an index to parent block in vector *bvp_state.blocks*
/// * `dimensions` are dimenisons of volume / parent block
/// * `block_dimensions` are dimensions of blocks inside the parent block
/// * `format_i` is an index to a format in vector *bvp_state.formats*
/// * `bvp_state` is a state tracking object for single BVP file
fn volume2block(parent_block_i: usize, dimensions: Vector3<u32>,
    block_dimensions: Vector3<u32>, format_i: usize, encoding: CompressionType, bvp_state: &mut BVPFile) -> Result<(), String> {
    
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

                // Check if block with the same data exists.
                let exists;
                let mut block_id = 0;
                match bvp_state.block_map.get(&block_hash) {
                    Some(bi) => {
                        let hashed_block = &bvp_state.blocks[*bi];
                        if hashed_block.is_equal_data(&block_data) {
                            // Blocks are equal - use the one already stored
                            exists = true;
                            block_id = *bi;
                        } else {
                            exists = false;
                        }
                    },
                    None => {
                        exists = false;
                    }
                };

                if !exists {
                    // Schedule block for writing to file and store its index
                    block_id = bvp_state.blocks.len();
                    let block_url = format!("blocks/block_{}.raw", block_id);
                    bvp_state.block_map.insert(block_hash, block_id);
                    let mut new_block = Block::new(block.dimensions, Some(format_i), None);
                    new_block.encoding = Some(encoding.to_string());
                    new_block.format = bvp_state.blocks[parent_block_i].format;
                    new_block.data_url = Some(block_url.clone());
                    bvp_state.blocks.push(new_block);

                    let block_data = match encoding {
                        CompressionType::None => block_data,
                        CompressionType::LZ4S => {
                            compression::compress_lz4s(&block_data)?
                        },
                    };

                    bvp_state.files.push(File::new(block_url, Rc::new(block_data), None));
                }

                bvp_state.blocks[parent_block_i].placements.push(Placement::new(block_start, block_id.clone()));
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
    let root_block_index = 0;

    bvp_file.modalities.push(Modality::new(
        parameters.name.clone(), parameters.description.clone(), parameters.semantic_type,
        parameters.volume_scale, parameters.voxel_scale, root_block_index
    ));
    bvp_file.asset.author = parameters.author;
    bvp_file.asset.copyright = parameters.copyright;
    bvp_file.asset.acquisition_time = parameters.acquisition_time;
    bvp_file.asset.generator = Some("raw2bvp script".to_string());  // TODO: Change to more interesting name
    bvp_file.asset.name = parameters.name;
    bvp_file.asset.description = parameters.description;

    // Create volume root block to populate with smaller blocks
    let parent_block = Block::new(parameters.dimensions, Some(root_block_index), Some(input_data));
    bvp_file.blocks.push(parent_block);
    
    volume2block(0, parameters.dimensions, parameters.block_dimensions, root_block_index, parameters.compression, &mut bvp_file)?;
    
    let time = chrono::offset::Utc::now();
    bvp_file.asset.creation_time = Some(time.timestamp().to_string()); // IN ISO format!!!
    bvp_file.files.push(File::new("manifest.json".to_string(), Rc::new(bvp_file.to_manifest()?), Some("application/json".to_string())));
    
    match &parameters.archive {
        archives::ArchiveEnum::SAF => {
            let saf = to_saf_archive(&bvp_file.files)?;
            match fs::write(parameters.output_file, saf) {
                Ok(_) => (),
                Err(e) => {
                    return Err(format!("Error outputing file: {}", e));
                }
            };
        },
        archives::ArchiveEnum::None => {
            for file in &bvp_file.files {
                file.write()?;
            }
        },
        _ => {
            println!("Not supported, exporting as raw files...");
            for file in &bvp_file.files {
                file.write()?;
            }
        }
    }

    return Ok(());
}