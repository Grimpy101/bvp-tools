mod raw_to_bvp;
mod arguments;

use std::env;

use crate::raw_to_bvp::{raw_to_bvp_parallel, raw_to_bvp_sequential};


fn main() -> Result<(), String> {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() < 2 {
        return Err("Missing JSON config file".to_string());
    }

    // let time_sequential_start = Instant::now();
    raw_to_bvp_sequential(&arguments[1])?;
    // println!(
    //     "Sequential execution time: {:.5}",
    //     time_sequential_start.elapsed().as_secs_f64()
    // );

    // let time_parallel_start = Instant::now();
    // raw_to_bvp_parallel(&arguments[1])?;
    // println!(
    //     "Parallel execution time: {:.5}",
    //     time_parallel_start.elapsed().as_secs_f64()
    // );

    return Ok(());
}

