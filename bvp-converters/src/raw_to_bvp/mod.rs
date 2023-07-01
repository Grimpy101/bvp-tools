mod parallel;
mod sequential;

use std::fs;
pub use parallel::raw_to_bvp_parallel;
//pub use sequential::raw_to_bvp_sequential;

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
