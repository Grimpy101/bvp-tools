use std::collections::HashMap;

use tinyjson::JsonValue;

use crate::{placement::Placement, formats::Format, vector3::Vector3};

pub struct Block {
    pub dimensions: Vector3<u32>,
    pub placements: Vec<Placement>,
    pub format: Option<usize>,
    pub data: Option<Vec<u8>>,
    pub data_url: Option<String>,
    encoding: Option<String>
}

impl Block {
    pub fn new(dimensions: Vector3<u32>, format: Option<usize>, data: Option<Vec<u8>>) -> Self {
        return Block {
            dimensions,
            placements: Vec::new(),
            format,
            data,
            encoding: None,
            data_url: None
        }
    }

    pub fn _set_data_in_range(&mut self, offset: Vector3<u32>, block: Block, format: &Format) -> Result<(), String> {
        let start = offset;
        let end = offset + block.dimensions;
        let extent = end - start;

        if self.format != block.format {
            return Err("Formats of the blocks do not match".to_string());
        }
        if extent.is_any_lt(Vector3::from_xyz(0, 0, 0)) {
            return Err("Start is greater than end".to_string());
        }
        if start.is_any_lt(Vector3::from_xyz(0, 0, 0)) {
            return Err("Start is out of bounds".to_string());
        }
        if end.is_any_gt(self.dimensions) {
            return Err("End is out of bounds".to_string());
        }

        let microblock_dimensions = format.microblock_dimensions;
        if start.is_any_div(&microblock_dimensions) {
            return Err("Block is not on microblock boundary".to_string());
        }
        if extent.is_any_div(&microblock_dimensions) {
            return Err("Block cannot contain whole microblocks".to_string());
        }

        let microblock_size = format.microblock_size;
        let microblock_start = (start / microblock_dimensions).to_u32();
        let microblock_amount_in_range = (extent / microblock_dimensions).to_u32();
        let microblock_amount_in_block = (self.dimensions / microblock_dimensions).to_u32();

        let src_bytes = match &block.data {
            Some(v) => v,
            None => return Err("Block does not have data".to_string()),
        };
        let mut dest_bytes = Vec::new();

        for x in 0..microblock_amount_in_range.x {
            for y in 0..microblock_amount_in_range.y {
                for z in 0..microblock_amount_in_range.z {
                    let local_microblock_index = Vector3::from_xyz(x, y, z);
                    let global_microblock_index = local_microblock_index + microblock_start;
                    let src_microblock_index = Vector3::linear_index(global_microblock_index, microblock_amount_in_block);
                    let dest_microblock_index = Vector3::linear_index(local_microblock_index, microblock_amount_in_range);

                    for i in 0..microblock_size {
                        let dest_index = i as usize + dest_microblock_index * microblock_size as usize;
                        let src_index = i as usize + src_microblock_index * microblock_size as usize;
                        dest_bytes[dest_index] = src_bytes[src_index];
                    }
                }
            }
        }

        self.data = Some(dest_bytes);

        return Ok(());
    }

    pub fn get_data_in_range(&self, start: Vector3<u32>, end: Vector3<u32>, format: &Format) -> Result<Block, String> {
        let extent = end - start;
        if extent.is_any_lt(Vector3::from_xyz(0, 0, 0)) {
            return Err("Start is greater than end".to_string());
        }
        if start.is_any_lt(Vector3::from_xyz(0, 0, 0)) {
            return Err("Start is out of bounds".to_string());
        }
        if end.is_any_gt(self.dimensions) {
            return Err("End is out of bounds".to_string());
        }

        let microblock_dimensions = format.microblock_dimensions;
        if start.is_any_div(&microblock_dimensions) {
            return Err("Block is not on microblock boundary".to_string());
        }
        if extent.is_any_div(&microblock_dimensions) {
            return Err("Block cannot contain whole microblocks".to_string());
        }

        let microblock_size = format.microblock_size;
        let microblock_start = (start / microblock_dimensions).to_u32();
        let microblock_amount_in_range = (extent / microblock_dimensions).to_u32();
        let microblock_amount_in_block = (self.dimensions / microblock_dimensions).to_u32();

        let mut block = Block::new(extent, self.format, None);
        let src_bytes = match &self.data {
            Some(v) => v,
            None => {
                return Err("Block does not have data".to_string())
            },
        };
        let dest_vec_size = format.count_space(extent) as usize;
        let mut dest_bytes = Vec::with_capacity(dest_vec_size);
        unsafe { dest_bytes.set_len(dest_vec_size); }
        
        for x in 0..microblock_amount_in_range.x {
            for y in 0..microblock_amount_in_range.y {
                for z in 0..microblock_amount_in_range.z {
                    let local_microblock_index = Vector3::from_xyz(x, y, z);
                    let global_microblock_index = local_microblock_index + microblock_start;
                    let src_microblock_index = Vector3::linear_index(global_microblock_index, microblock_amount_in_block);
                    let dest_microblock_index = Vector3::linear_index(local_microblock_index, microblock_amount_in_range);
                    for i in 0..microblock_size {
                        let dest_index = i as usize + dest_microblock_index * microblock_size as usize;
                        let src_index = i as usize + src_microblock_index * microblock_size as usize;
                        dest_bytes[dest_index] = src_bytes[src_index];
                    }
                }
            }
        }

        block.data = Some(dest_bytes);
        return Ok(block);
    }

    pub fn to_hashmap(&self) -> HashMap<String, JsonValue> {
        let mut hm = HashMap::new();
        let mut placements = Vec::new();
        for placement in &self.placements {
            placements.push(placement.to_hashmap().into());
        }
        hm.insert("placements".to_string(), placements.into());
        hm.insert("dimensions".to_string(), self.dimensions.to_f64_vec().into());
        if self.format.is_some() {
            hm.insert("format".to_string(), (self.format.unwrap() as f64).into());
        }
        if self.data_url.is_some() {
            hm.insert("data".to_string(), self.data_url.as_ref().unwrap().clone().into());
        }
        if self.encoding.is_some() {
            hm.insert("encoding".to_string(), self.encoding.as_ref().unwrap().clone().into());
        }

        return hm;
    }

    pub fn _to_string(&self) -> String {
        let dims = self.dimensions;
        let plc = self.placements.len();
        let mut data = 0;
        if self.data.is_some() {
            data = self.data.as_ref().unwrap().len();
        }
        return format!("Block: dims {}, plc {}, data {}", dims, plc, data);
    }
}