use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::available_parallelism;

use crossbeam::{channel, scope};
use crossbeam::channel::{Receiver, Sender};
use crossbeam::thread::Scope;
use itertools::iproduct;
use xxhash_rust::xxh3;
use zip::{CompressionMethod, ZipWriter};
use zip::write::FileOptions;

use bvp::block::Block;
use bvp::bvpfile::BVPFile;
use bvp::compressions::CompressionType;
use bvp::file::File;
use bvp::modality::Modality;
use bvp::placement::Placement;
use bvp::vector3::Vector3;
use crate::arguments;
use crate::arguments::Parameters;
use crate::raw_to_bvp::read_input_file;


struct StageOnePipelineResult {
    pub block_start: Vector3<u32>,
    pub block_end: Vector3<u32>,

    pub format_index: usize,
    pub parent_block_index: usize,
}

struct StageTwoPipelineResult {
    file_to_write: File,
}


/*
 * Pipeline, stage 1
 */

/// Spawn stage one thread for the pipeline.
///
/// This stage has a single thread that parses the input file and parameters and
/// generates all the block ranges we need to parse the raw data into smaller blocks.
///
/// It then sends the "work packets" through the provided `Sender`.
fn spawn_stage_1<'parameters: 'scope_env, 'scope, 'scope_env: 'scope>(
    scope: &'scope Scope<'scope_env>,
    stage_one_result_channel_tx: Sender<StageOnePipelineResult>,
    parameters: &'parameters Parameters,
    root_block_format: usize,
) {
    scope.spawn(move |_| {
        let dimensions = parameters.dimensions;
        let block_dimensions = parameters.block_dimensions;

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


/*
 * Pipeline, stage 2
 */

/// Run a worker for the stage two of the pipeline.
///
/// This stage uses the ranges provided by first stage and generates smaller blocks of data,
/// performs deduplication and compresses them.
///
/// Block hashes, blocks themselves and placements are stored in three locked variables,
/// `bvp_shared_block_map`, `bvp_shared_block_vec` and `bvp_shared_parent_placements_vec`.
/// This data will be needed after the pipeline concludes to finalize the `BVPFile` instance
/// before writing the manifest.
fn run_stage_2_worker(
    stage_one_result_channel_rx: Arc<Receiver<StageOnePipelineResult>>,
    stage_two_result_queue_tx: Arc<Sender<StageTwoPipelineResult>>,
    bvp_shared_block_map: Arc<Mutex<HashMap<u64, usize>>>,
    bvp_shared_block_vec: Arc<Mutex<Vec<Block>>>,
    bvp_shared_parent_placements_vec: Arc<Mutex<Vec<Placement>>>,
    bvp_file: Arc<BVPFile>,
    encoding: CompressionType,
) -> Result<(), String> {
    // Parses blocks to be saved in separate files (and performs deduplication).
    loop {
        let prepared_work = match stage_one_result_channel_rx.recv() {
            Ok(work) => work,
            Err(_) => {
                // This happens when the channel sender has been dropped and there is
                // no more work to receive.
                break;
            }
        };

        let format = &bvp_file.formats[prepared_work.format_index];
        let block = bvp_file.blocks[prepared_work.parent_block_index]
            .get_data_in_range(
                prepared_work.block_start,
                prepared_work.block_end,
                format,
            )
            .map_err(|err| err.to_string())?;

        let block_data = block.data
            .ok_or_else(|| String::from("Block does not have data!"))?;
        let block_format_index = block.format;

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
            let same_hash_block_data: &Block = locked_blocks_vec
                .get(*same_hash_block_id)
                .expect("Block with given index did not exist in vector.");

            if same_hash_block_data.is_equal_data(&block_data) {
                // Real collision, we can deduplicate and don't need to write another file.
                {
                    let mut locked_placements = bvp_shared_parent_placements_vec.lock()
                        .expect("Some thread panicked while holding shared Placements vec.");

                    locked_placements.push(Placement::new(
                        prepared_work.block_start,
                        *same_hash_block_id,
                    ));
                }

                continue;
            }
        }

        // No collision and no possibility of deduplication,
        // schedule block for writing to file and store its index.
        let block_id = locked_blocks_vec.len() + 1;
        let block_url = format!("blocks/block_{}.raw", block_id);

        locked_blocks_map.insert(block_data_hash, block_id);

        let mut new_block = Block::new(
            block_id,
            block.dimensions,
            Some(prepared_work.format_index),
            None,
        );

        new_block.encoding = Some(encoding);
        new_block.format = block_format_index;
        new_block.data_url = Some(block_url.clone());

        locked_blocks_vec.push(new_block);

        drop(locked_blocks_vec);
        drop(locked_blocks_map);
        /*
         * Here ends the locked segment.
         */

        let compressed_block_data = encoding.compress(block_data);

        {
            let mut locked_placements = bvp_shared_parent_placements_vec.lock()
                .expect("Some thread panicked while holding shared Placements vec.");

            locked_placements.push(Placement::new(
                prepared_work.block_start,
                block_id,
            ));
        }

        stage_two_result_queue_tx.send(StageTwoPipelineResult {
            file_to_write: File::new(
                block_url,
                Arc::new(compressed_block_data),
                None,
            )
        })
            .expect("Stage two could not send! Did stage three drop receiver?");
    }

    Ok(())
}

/// Spawn stage two threads for the pipeline.
///
/// See `run_stage_2_worker` for more information.
fn spawn_stage_2<'scope, 'scope_env: 'scope>(
    number_of_workers: usize,
    scope: &'scope Scope<'scope_env>,
    stage_one_result_channel_rx: Arc<Receiver<StageOnePipelineResult>>,
    stage_two_result_queue_tx: Arc<Sender<StageTwoPipelineResult>>,
    bvp_shared_block_map: Arc<Mutex<HashMap<u64, usize>>>,
    bvp_shared_block_vec: Arc<Mutex<Vec<Block>>>,
    bvp_shared_parent_placements_vec: Arc<Mutex<Vec<Placement>>>,
    bvp_file: Arc<BVPFile>,
    encoding: CompressionType,
) {
    for _ in 0..number_of_workers {
        let stage_one_result_channel_rx_clone = stage_one_result_channel_rx.clone();
        let stage_two_result_queue_tx_clone = stage_two_result_queue_tx.clone();
        let bvp_shared_block_map_clone = bvp_shared_block_map.clone();
        let bvp_shared_block_vec_clone = bvp_shared_block_vec.clone();
        let bvp_shared_parent_placements_vec_clone = bvp_shared_parent_placements_vec.clone();
        let bvp_file_clone = bvp_file.clone();

        scope.spawn(move |_| {
            run_stage_2_worker(
                stage_one_result_channel_rx_clone,
                stage_two_result_queue_tx_clone,
                bvp_shared_block_map_clone,
                bvp_shared_block_vec_clone,
                bvp_shared_parent_placements_vec_clone,
                bvp_file_clone,
                encoding,
            )
        });
    }
}


/*
 * Stage 3
 */
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
                .compression_method(CompressionMethod::Stored),
        )
            .map_err(|err| err.to_string())?;

        Ok(Self {
            zip_writer,
        })
    }

    fn append_block_file(&mut self, file: File) -> Result<(), String> {
        self.zip_writer.start_file(
            file.name,
            FileOptions::default()
                .compression_method(CompressionMethod::Stored),
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
                .compression_method(CompressionMethod::Stored),
        )
            .map_err(|err| err.to_string())?;

        self.zip_writer
            .write_all(manifest_file.data.as_slice())
            .map_err(|err| err.to_string())?;

        let mut inner_file = self.zip_writer
            .finish()
            .map_err(|err| err.to_string())?;

        inner_file.flush()
            .map_err(|err| err.to_string())?;

        Ok(())
    }
}

/// Run a worker for the stage three of the pipeline.
///
/// This stage receives parsed data blocks from the second stage and
/// writes them into the .bvp (ZIP) file.
fn run_stage_3_worker(
    stage_two_result_queue_rx: Receiver<StageTwoPipelineResult>,
    zip_writer: &mut StreamingZIPArchiveWriter,
) -> Result<(), String> {
    // Receive queued files to write and write them to disk as the requests are coming in.
    loop {
        let stage_two_work = match stage_two_result_queue_rx.recv() {
            Ok(work) => work,
            Err(_) => {
                // This happens when the channel sender has been dropped and there is
                // no more work to receive.
                break;
            }
        };

        zip_writer
            .append_block_file(stage_two_work.file_to_write)?;
    }

    Ok(())
}

/// Spawn stage three worker for the pipeline.
///
/// See `run_stage_3_worker` for more information.
fn spawn_stage_3<'zip_writer: 'scope_env, 'scope, 'scope_env: 'scope>(
    scope: &'scope Scope<'scope_env>,
    stage_two_result_queue_rx: Receiver<StageTwoPipelineResult>,
    zip_writer: &'zip_writer mut StreamingZIPArchiveWriter,
) -> Result<(), String> {
    scope.spawn(move |_| {
        run_stage_3_worker(
            stage_two_result_queue_rx,
            zip_writer,
        )
    });

    Ok(())
}


/*
 * Other
 */

fn finalize_bvp_zip_file(
    zip_writer: StreamingZIPArchiveWriter,
    mut bvp_file: BVPFile,
    bvp_block_map: HashMap<u64, usize>,
    bvp_block_vec: Vec<Block>,
    root_block_index: usize,
    bvp_root_block_placements_vec: Vec<Placement>,
    parameters: &Parameters,
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

    bvp_file.block_map = bvp_block_map;
    bvp_file.blocks.extend(bvp_block_vec);
    bvp_file.blocks[root_block_index].placements = bvp_root_block_placements_vec;


    let manifest_data = bvp_file.to_manifest()?;
    let manifest_file = File::new(
        "manifest.json".to_string(),
        Arc::new(manifest_data),
        Some("application/json".to_string()),
    );

    zip_writer.finish(manifest_file)?;

    Ok(())
}

/*
 * Entry function
 */

pub fn raw_to_bvp_parallel(
    config_file_path: &str
) -> Result<(), String> {
    // Detect available cores on the system.
    let stage_two_worker_count: usize = available_parallelism()
        .map_err(|err| err.to_string())?
        .into();

    // Set up inter-stage channels/queues/maps/vectors.
    let (stage_one_result_channel_tx, stage_one_result_channel_rx) =
        channel::unbounded::<StageOnePipelineResult>();
    let stage_one_result_channel_rx_arc = Arc::new(stage_one_result_channel_rx);

    let (stage_two_result_channel_tx, stage_two_result_channel_rx) =
        channel::unbounded::<StageTwoPipelineResult>();
    let stage_two_result_channel_tx_arc = Arc::new(stage_two_result_channel_tx);

    let bvp_shared_block_map: Arc<Mutex<HashMap<u64, usize>>> = Arc::new(Mutex::new(HashMap::new()));
    let bvp_shared_block_vec: Arc<Mutex<Vec<Block>>> = Arc::new(Mutex::new(Vec::new()));
    let bvp_shared_root_placements_vec: Arc<Mutex<Vec<Placement>>> = Arc::new(Mutex::new(Vec::new()));

    // Parse parameters and read input file.
    let parameters = arguments::parse_config(config_file_path)
        .map_err(|err| err.to_string())?;

    let raw_input_data = read_input_file(&parameters.input_file)?;

    // Initialize BVPFile instance (as much as we need to before going parallel).
    let mut bvp = BVPFile::new();

    // FIXME Not sure how this `format` works, used constant value
    //       in `root_block_index` from sequential implementation.
    let root_block_format_index: usize = 0;
    let root_block_index = bvp.blocks.len();
    let root_block = Block::new(
        root_block_index,
        parameters.dimensions,
        Some(root_block_format_index),
        Some(raw_input_data),
    );

    bvp.formats.push(parameters.input_format.clone());
    bvp.blocks.push(root_block);

    // The pipeline will now have read-only access to the BVPFile.
    // After the pipeline concludes, we'll unwrap the `Arc` and finish adding any
    // missing fields before finalizing the .bvp file.
    let bvp_arc = Arc::new(bvp);

    // Initialize writer for ZIP files.
    let mut zip_writer = StreamingZIPArchiveWriter::new(
        parameters.output_file.clone()
    )
        .map_err(|err| err.to_string())?;

    // Spawn pipeline stages and wait for finish.
    // Pipeline is made out of three stages:
    //   - First stage (single thread) parses the input file and parameters and
    //     generates all the block ranges we need to parse the raw data into smaller blocks.
    //   - Second stage (as many threads as cores) uses the ranges provided by first stage
    //     and generates smaller blocks of data, performs deduplication and compresses the data.
    //   - Third stage (single thread) receives parsed data blocks from the second stage and
    //     writes them into the .bvp file.
    // The pipeline is constructed using a thread scope from `crossbeam` - stages run in parallel
    // and each stage shuts down when it has completed all the work the previous stage can provide.
    scope(|scope| {
        // Stage 1 (parse input file and generate block ranges)
        spawn_stage_1(
            scope,
            stage_one_result_channel_tx,
            &parameters,
            root_block_format_index,
        );

        // Stage 2 (parse smaller blocks, deduplicate and compress)
        spawn_stage_2(
            stage_two_worker_count,
            scope,
            stage_one_result_channel_rx_arc,
            stage_two_result_channel_tx_arc,
            bvp_shared_block_map.clone(),
            bvp_shared_block_vec.clone(),
            bvp_shared_root_placements_vec.clone(),
            bvp_arc.clone(),
            parameters.compression,
        );

        // Stage 3 (write queued files to zip)
        spawn_stage_3(
            scope,
            stage_two_result_channel_rx,
            &mut zip_writer,
        )?;

        Ok::<(), String>(())
    })
        .map_err(|_| String::from("Scope failed to execute."))??;

    // Unwrap `Arc`s and `Mutex`es that must, at this point, have only one strong reference
    // and no other threads can access them. We could technically keep them as-is,
    // but unwrapping into original types allows us cleaner access.
    let bvp_file = Arc::try_unwrap(bvp_arc)
        .expect("BUG: Something is holding a strong reference somehow.");

    let bvp_block_map = Arc::try_unwrap(bvp_shared_block_map)
        .expect("BUG: Something is still holding a strong reference.")
        .into_inner()
        .expect("Could not lock shared block map, some thread panicked while holding the lock.");

    let bvp_block_vec = Arc::try_unwrap(bvp_shared_block_vec)
        .expect("BUG: Something is still holding a strong reference.")
        .into_inner()
        .expect("Could not lock shared block vec, some thread panicked while holding the lock.");

    let bvp_root_placements_vec = Arc::try_unwrap(bvp_shared_root_placements_vec)
        .expect("BUG: Something is still holding a strong reference.")
        .into_inner()
        .expect("Could not lock shared root placements vec, some thread panicked while holding the lock.");

    // Finalize BVPFile, generate and write the manifest and close the zip file writer.
    finalize_bvp_zip_file(
        zip_writer,
        bvp_file,
        bvp_block_map,
        bvp_block_vec,
        root_block_index,
        bvp_root_placements_vec,
        &parameters,
    )?;

    Ok(())
}

