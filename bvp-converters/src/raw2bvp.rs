use std::{rc::Rc, env, fs};
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use crossbeam::queue::ArrayQueue;
use crossbeam::{channel, scope};
use crossbeam::channel::{Receiver, RecvError, Sender};
use crossbeam::sync::ShardedLock;
use crossbeam::thread::Scope;
use itertools::iproduct;

use bvp::bvpfile::BVPFile;
use bvp::compressions::CompressionType;
use bvp::file::File;
use bvp::modality::Modality;
use xxhash_rust::xxh3;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

use bvp::block::Block;
use bvp::atomic_token::AtomicBoolToken;
use bvp::errors::ArchiveError;
use bvp::placement::Placement;
use bvp::vector3::Vector3;
use crate::arguments::Parameters;

mod arguments;

/*
 * SEQUENTIAL IMPLEMENTATION
 */

/// Iterates through blocks of data in volume,
/// schedules unique data for writing to file,
/// and reference data through placements
/// 
/// * `parent_block_i` is an index to parent block in vector *bvp_state.blocks*
/// * `dimensions` are dimensions of volume / parent block
/// * `block_dimensions` are dimensions of blocks inside the parent block
/// * `format_i` is an index to a format in vector *bvp_state.formats*
/// * `bvp_state` is a state tracking object for single BVP file
fn volume2block(parent_block_index: usize, dimensions: Vector3<u32>, block_dimensions: Vector3<u32>, format_index: usize, encoding: CompressionType, bvp_state: &mut BVPFile) -> Result<(), String> {
    
    let block_count = (dimensions / block_dimensions).ceil();
    let format = &bvp_state.formats[format_index];

    for x in 0..block_count.x {
        for y in 0..block_count.y {
            for z in 0..block_count.z {
                let block_start = block_dimensions * Vector3::from_xyz(x, y, z);
                let block_end = (block_start + block_dimensions).min(&dimensions);

                let block = (&bvp_state.blocks[parent_block_index]).get_data_in_range(block_start, block_end, format).map_err(|x| format!("{}", x))?;
                let block_data = match block.data {
                    Some(d) => d,
                    None => {
                        return Err("Block does not have data".to_string());
                    }
                };
                let block_hash = xxh3::xxh3_64(&block_data[..]);

                // Check if block with the same data exists.
                // Data hashes are compared, since this is faster.
                // If hashes are equal, raw data is compared in case of collisions.
                let exists;
                let mut block_id = 0;
                match bvp_state.block_map.get(&block_hash) {
                    Some(bi) => {
                        let hashed_block = &bvp_state.blocks[*bi];
                        if hashed_block.is_equal_data(&block_data) {
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
                    let mut new_block = Block::new(block_id, block.dimensions, Some(format_index), None);
                    new_block.encoding = Some(encoding);
                    new_block.format = bvp_state.blocks[parent_block_index].format;
                    new_block.data_url = Some(block_url.clone());
                    bvp_state.blocks.push(new_block);

                    let block_data = encoding.compress(block_data);

                    bvp_state.files.push(File::new(block_url, Rc::new(block_data), None));
                }

                bvp_state.blocks[parent_block_index].placements.push(Placement::new(block_start, block_id.clone()));
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

fn raw_to_bvp_sequential(
    config_file_path: &str
) -> Result<(), String> {
    let parameters = arguments::parse_config(config_file_path).map_err(|x| format!("{}", x))?;
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
    let parent_block_index = bvp_file.blocks.len();
    let parent_block = Block::new(parent_block_index, parameters.dimensions, Some(root_block_index), Some(input_data));
    bvp_file.blocks.push(parent_block);

    volume2block(0, parameters.dimensions, parameters.block_dimensions, root_block_index, parameters.compression, &mut bvp_file)?;

    let time = chrono::offset::Utc::now();
    bvp_file.asset.creation_time = Some(time.timestamp().to_string()); // IN ISO format!!!
    bvp_file.files.push(File::new("manifest.json".to_string(), Rc::new(bvp_file.to_manifest()?), Some("application/json".to_string())));

    parameters.archive.write_files(&bvp_file.files, parameters.output_file).map_err(|x| format!("{}", x))?;

    Ok(())
}


/*
 * PARALLEL IMPLEMENTATION
 */

struct StageOnePipelineResult {
    pub block_start: Vector3<u32>,
    pub block_end: Vector3<u32>,

    pub format_index: usize,
    pub parent_block_index: usize,
}

fn spawn_stage_1<'scope>(
    scope: &'scope Scope<'scope>,
    stage_one_result_channel_tx: Sender<StageOnePipelineResult>,
    bvp_file: Arc<BVPFile>,
    parameters: &Parameters,
) {
    scope.spawn(|_| {
        bvp_file.formats.push(parameters.input_format.clone());

        let raw_input_data = read_input_file(&parameters.input_file)?;

        // FIXME Not sure how this `format` works, used constant value
        //       in `root_block_index` from sequential implementation.
        let root_block_format: usize = 0;

        let parent_block_index = bvp_file.blocks.len();
        let parent_block = Block::new(
            parent_block_index,
            parameters.dimensions,
            Some(root_block_format),
            Some(raw_input_data),
        );

        bvp_file.blocks.push(parent_block);


        let dimensions = parameters.dimensions;
        let block_dimensions = parameters.block_dimensions;
        let compression = parameters.compression;

        let block_count = (dimensions / block_dimensions).ceil();


        for (x, y, z) in iproduct!(0..block_count.x, 0..block_count.y, 0..block_count.z) {
            let block_start = block_dimensions * Vector3::from_xyz(x, y, z);
            let block_end = (block_start + block_dimensions).min(&dimensions);

            stage_one_result_channel_tx.send(StageOnePipelineResult {
                block_start,
                block_end,
                format_index: root_block_format,
                parent_block_index: 0,
            })
                .expect("Stage one could not send result. Did stage two drop receivers?");
        }
    });
}


enum BlockEncodingResult {
    AlreadyExists {
        placement: Placement,
    },
    NewFile {
        file: File,
        placement: Placement,
    }
}

struct StageTwoPipelineResult {
    encoding_result: BlockEncodingResult,
}


fn spawn_stage_2<'scope>(
    scope: &'scope Scope<'scope>,
    stage_one_result_channel_rx: Arc<Receiver<StageOnePipelineResult>>,
    stage_two_result_queue_tx: Arc<Sender<StageTwoPipelineResult>>,
    bvp_shared_block_map: Arc<Mutex<HashMap<u64, usize>>>,
    bvp_shared_block_vec: Arc<Mutex<Vec<Block>>>,
    bvp_file: Arc<BVPFile>,
    encoding: CompressionType,
) {
    // TODO spawn more than one thread
    scope.spawn(|_| {
        // Pseudocode: parse blocks to be saved in separate files

        loop {
            let prepared_work = match stage_one_result_channel_rx.recv() {
                Ok(work) => work,
                Err(_) => {
                    // This happens when the channel sender has been dropped and there is
                    // no more work to receive.
                    break;
                }
            };

            let (block_data, format_index) = {
                let locked_bvp = bvp_file.read()
                    .expect("BUG: already held by current thread.");

                let format = &locked_bvp.formats[prepared_work.format_index];
                let block = locked_bvp.blocks[prepared_work.parent_block_index]
                    .get_data_in_range(
                        prepared_work.block_start,
                        prepared_work.block_end,
                        format,
                    )
                    .map_err(|err| err.to_string())?;

                (
                    block.data
                        .ok_or_else(|| String::from("Block does not have data!"))?,
                    block.format
                )
            };

            let block_data_hash = xxh3::xxh3_64(block_data.as_slice());

            /*
             * Here begins a locked segment (only one thread at a time), which is required
             * if we want to preserve deduplication.
             */
            let mut locked_blocks_map = bvp_shared_block_map.lock()
                .expect("Shared Block index map Mutex lock has been poisoned!");
            let mut locked_blocks_vec = bvp_shared_block_vec.lock()
                .expect("Shared Block vector Mutex lock has been poisoned!");

            // Check if block with the same hash exists.
            // If a hash collision is found, compare the raw data before assuming
            // the blocks are actually the same.
            if let Some(same_hash_block_id) = locked_blocks_map.get(&block_data_hash)
            {
                // TODO Maybe replace this with sharded lock for better reads
                let same_hash_block_data: &Block = bvp_shared_block_vec
                    .lock()
                    .expect("Block vec Mutex lock has been poisoned!")
                    .get(same_hash_block_id)
                    .expect("Block with given index did not exist in vector.");

                if same_hash_block_data.is_equal_data(&block_data) {
                    // Real collision, we can deduplicate.
                    stage_two_result_queue_tx.send(StageTwoPipelineResult {
                        encoding_result: BlockEncodingResult::AlreadyExists {
                            placement: Placement::new(
                                prepared_work.block_start,
                                *same_hash_block_id
                            )
                        }
                    })
                        .expect("Stage two result channel could not send! Did stage 3 drop the receiver?");

                    continue;
                }
            }

            // No collision and no possibility of deduplication,
            // schedule block for writing to file and store its index.
            let block_id = locked_blocks_vec.len();
            let block_url = format!("blocks/block_{}.raw", block_id);

            locked_blocks_map.insert(block_data_hash, block_id);

            let mut new_block = Block::new(
                block_id,
                block.dimensions,
                Some(prepared_work.format_index),
                None,
            );

            new_block.encoding = Some(encoding);
            new_block.format = format_index;
            new_block.data_url = Some(block_url.clone());

            locked_blocks_vec.push(new_block);

            drop(locked_blocks_vec);
            drop(locked_blocks_map);
            /*
             * Here ends the locked segment.
             */

            let compressed_block_data = encoding.compress(block_data);

            stage_two_result_queue_tx.send(StageTwoPipelineResult {
                encoding_result: BlockEncodingResult::NewFile {
                    file: File::new(
                        block_url,
                        Rc::new(compressed_block_data),
                        None,
                    ),
                    placement: Placement::new(
                        prepared_work.block_start,
                        block_id,
                    ),
                }
            })
                .expect("Stage two could not send! Did stage three drop receiver?");

        }

        if !stage_two_finished_token.is_true() {
            stage_two_finished_token.set_true();
        }
    });
}

trait StreamingArchiveWriter {
    fn append_block_file(&mut self, file: File) -> Result<(), String>;
    fn finish(self, manifest_file: File) -> Result<(), String>;
}

pub struct StreamingZIPArchiveWriter {
    zip_writer: ZipWriter<fs::File>,
}

impl StreamingZIPArchiveWriter {
    pub fn new<P: AsRef<Path>>(output_file_path: P) -> Result<Self, String> {
        let output_file = fs::File::create(output_file_path.as_ref())
            .map_err(|e| e.to_string())?;

        let mut zip_writer = ZipWriter::new(output_file);

        zip_writer.add_directory(
            "blocks",
            FileOptions::default()
                .compression_method(CompressionMethod::Stored)
        )
            .map_err(|err| err.to_string())?;

        Ok(Self {
            zip_writer,
        })
    }
}

impl StreamingArchiveWriter for StreamingZIPArchiveWriter {
    fn append_block_file(&mut self, file: File) -> Result<(), String> {
        self.zip_writer.start_file(
            file.name,
            FileOptions::default()
                .compression_method(CompressionMethod::Stored)
        )
            .map_err(|err| err.to_string())?;

        self.zip_writer
            .write_all(file.data.as_slice())
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    fn finish(mut self, manifest_file: File) -> Result<(), String> {
        self.zip_writer.start_file(
            manifest_file.name,
            FileOptions::default()
                .compression_method(CompressionMethod::Stored)
        )
            .map_err(|err| err.to_string())?;

        self.zip_writer
            .write_all(manifest_file.data.as_slice())
            .map_err(|err| err.to_string())?;


        self.output_file.flush()
            .map_err(|e| e.to_string())
    }
}


fn spawn_stage_3<'scope, P: AsRef<Path>>(
    scope: &'scope Scope<'scope>,
    stage_two_result_queue_rx: Receiver<StageTwoPipelineResult>,
    zip_writer: &mut StreamingZIPArchiveWriter,
) -> Result<(), String> {

    scope.spawn(|_| {
        // Pseudocode: receive parsed chunks, write to disk as the requests are coming in.
        // when channel is dry, write the manifest
        loop {
            let stage_two_work = match stage_two_result_queue_rx.recv() {
                Ok(work) => work,
                Err(_) => {
                    // This happens when the channel sender has been dropped and there is
                    // no more work to receive.
                    break;
                }
            };

            let placement = match stage_two_work.encoding_result {
                BlockEncodingResult::NewFile { file, placement } => {
                    zip_writer
                        .append_block_file(file)?;

                    placement
                }
                BlockEncodingResult::AlreadyExists { placement } => placement
            };

            // TODO placement?
        }

    });

    Ok(())
}

fn finalize_bvp_zip_file(
    mut zip_writer: StreamingZIPArchiveWriter,
    mut bvp_file: BVPFile,
) -> Result<(), String> {
    bvp_file.modalities.push(Modality::new(
        parameters.name.clone(),
        parameters.description.clone(),
        parameters.semantic_type.clone(),
        parameters.volume_scale,
        parameters.voxel_scale,
        root_block_index,
    ));

    bvp_file.asset.author = parameters.author.clone();
    bvp_file.asset.copyright = parameters.copyright.clone();
    bvp_file.asset.acquisition_time = parameters.acquisition_time.clone();
    bvp_file.asset.generator = Some("raw2bvp script".to_string());  // TODO: Change to more interesting name
    bvp_file.asset.name = parameters.name.clone();
    bvp_file.asset.description = parameters.description.clone();

    let time = chrono::offset::Utc::now();
    bvp_file.asset.creation_time = Some(time.timestamp().to_string()); // IN ISO format!!!

    bvp_file.block_map = blocks_map;
    bvp_file.blocks = blocks_vec;

    bvp_file.files.extend(bvp_file_list.into_iter());


    let manifest_data = bvp_file.to_manifest()?;
    let manifest_file = File::new(
        "manifest.json".to_string(),
        Rc::new(manifest_data),
        Some("application/json".to_string()),
    );

    zip_writer.finish(manifest_file)?;

    Ok(())
}

fn raw_to_bvp_parallel(
    config_file_path: &str
) -> Result<(), String> {
    // Set up inter-stage channels/queues.
    let (stage_one_result_channel_tx, stage_one_result_channel_rx) =
        channel::unbounded::<StageOnePipelineResult>();
    let stage_one_result_channel_rx_arc = Arc::new(stage_one_result_channel_rx);

    let (stage_two_result_channel_tx, stage_two_result_channel_rx) =
        channel::unbounded::<StageTwoPipelineResult>();
    let stage_two_result_channel_tx_arc = Arc::new(stage_two_result_channel_tx);

    let bvp_shared_block_map: Arc<Mutex<HashMap<u64, usize>>> = Arc::new(Mutex::new(HashMap::new()));
    let bvp_shared_block_vec: Arc<Mutex<Vec<Block>>> = Arc::new(Mutex::new(Vec::new()));

    let parameters = arguments::parse_config(config_file_path)
        .map_err(|err| err.to_string())?;

    let bvp_sharded_arc = Arc::new(BVPFile::new());

    let mut zip_writer = StreamingZIPArchiveWriter::new(parameters.output_file)
        .map_err(|err| err.to_string())?;

    scope(|scope| {
        // Stage 1
        spawn_stage_1(
            scope,
            stage_one_result_channel_tx,
            bvp_sharded_arc.clone()
        );

        // Stage 2
        spawn_stage_2(
            scope,
            stage_one_result_channel_rx_arc.clone(),
            stage_two_result_channel_tx_arc.clone(),
            bvp_shared_block_map.clone(),
            bvp_shared_block_vec.clone(),
            bvp_sharded_arc.clone(),
            parameters.compression,
        );

        // Stage 3
        spawn_stage_3(
            scope,
            stage_two_result_channel_rx,
            &mut zip_writer,
        )?;
    })
        .map_err(|err| String::from(err))?;


    let bvp_file = Arc::try_unwrap(bvp_sharded_arc)
        .expect("BUG: Something is holding a strong reference somehow.");

    finalize_bvp_zip_file(
        zip_writer,
        bvp_file
    )?;

    Ok(())
}




/*
 * MAIN
 */

fn main() -> Result<(), String> {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() < 2 {
        return Err("Missing JSON config file".to_string());
    }

    raw_to_bvp_sequential(&arguments[1])?;

    return Ok(());
}
