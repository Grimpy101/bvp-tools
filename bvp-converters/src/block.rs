use std::collections::HashMap;

use tinyjson::JsonValue;

use crate::{placement::Placement, formats::Format, vector3::Vector3, json_aux::{get_u32_from_json, get_string_from_json}, bvpfile::File, compression};

pub struct Block {
    pub dimensions: Vector3<u32>,
    pub placements: Vec<Placement>,
    pub format: Option<usize>,
    pub data: Option<Vec<u8>>,
    pub data_url: Option<String>,
    pub encoding: Option<String>
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

    pub fn set_data_in_range(&mut self, offset: Vector3<u32>, block: &Block, format: &Format) -> Result<(), String> {
        let start = offset;
        let end = offset + block.dimensions;
        let extent = end - start;
        if self.data.is_none() {
            return Err("Target block does not have data".to_string());
        }
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

        let src_original_len = format.count_space(block.dimensions) as usize;
        let mut decompressed_src = Vec::new();
        let src_bytes = match &block.data {
            Some(v) => {
                if block.encoding.is_none() {
                    v
                } else {
                    match block.encoding.as_ref().unwrap().as_str() {
                        "raw" => v,
                        "lz4s" => {
                            decompressed_src = compression::decompress_lz4s(&v, src_original_len);
                            &decompressed_src
                        },
                        _ => return Err("Unknown compression scheme".to_string())
                    }
                }
            },
            None => return Err("Block does not have data".to_string()),
        };
        let dest_bytes = self.data.as_mut().unwrap();

        for x in 0..microblock_amount_in_range.x {
            for y in 0..microblock_amount_in_range.y {
                for z in 0..microblock_amount_in_range.z {
                    let local_microblock_index = Vector3::from_xyz(x, y, z);
                    let global_microblock_index = local_microblock_index + microblock_start;
                    let src_microblock_index = Vector3::linear_index(local_microblock_index, microblock_amount_in_range);
                    let dest_microblock_index = Vector3::linear_index(global_microblock_index, microblock_amount_in_block);

                    for i in 0..microblock_size {
                        let dest_index = i as usize + dest_microblock_index * microblock_size as usize;
                        let src_index = i as usize + src_microblock_index * microblock_size as usize;
                        dest_bytes[dest_index] = src_bytes[src_index];
                    }
                }
            }
        }
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

    pub fn is_equal_data(&self, vec: &Vec<u8>) -> bool {
        if self.data.is_none() {
            return false;
        }
        let data = self.data.as_ref().unwrap();
        if data.len() != vec.len() {
            return false;
        }

        for i in 0..data.len() {
            if data[i] != vec[i] {
                return false;
            }
        }

        return true;
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

    // This here is probably not optimal...
    pub fn from_json(j: &JsonValue, files: &Vec<File>) -> Result<Self, String> {
        match j {
            JsonValue::Object(o) => {
                let dimensions = Vector3::<u32>::from_json(&o["dimensions"])?;
                let mut placements = Vec::new();
                match &o["placements"] {
                    JsonValue::Array(a) => {
                        for el in a {
                            placements.push(Placement::from_json(&el)?);
                        }
                    },
                    _ => {
                        return Err("Invalid JSON".to_string());
                    }
                };
                let mut block = Block {
                    dimensions,
                    placements,
                    format: None,
                    data: None,
                    data_url: None,
                    encoding: None
                };

                match o.get("format") {
                    Some(f) => {
                        let format = get_u32_from_json(f)? as usize;
                        block.format = Some(format);
                    },
                    None => ()
                }

                match o.get("data") {
                    Some(d) => {
                        let data_url = get_string_from_json(d)?;
                        // This is technically required by specification... TODO: Implement!
                        //let encoding = get_string_from_json(&o["encoding"])?;
                        
                        for file in files {
                            if file.name == data_url {
                                let data = file.data.to_vec();
                                block.data_url = Some(data_url);
                                //block.encoding = Some(encoding);
                                block.data = Some(data);
                                break;
                            }
                        }
                    },
                    None => ()
                }
                return Ok(block);
            },
            _ => ()
        }
        return Err("Not a valid JSON".to_string());
    }
}
