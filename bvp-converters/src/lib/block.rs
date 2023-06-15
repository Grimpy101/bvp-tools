use std::{collections::HashMap};

use tinyjson::JsonValue;

use crate::{placement::Placement, formats::Format, vector3::Vector3, json_aux::{get_u32_from_json, get_string_from_json}, file::File, errors::{BlockError, JsonError}, compressions::{CompressionType}};

pub struct Block {
    pub index: usize,
    pub dimensions: Vector3<u32>,
    pub placements: Vec<Placement>,
    pub format: Option<usize>,
    pub data: Option<Vec<u8>>,
    pub data_url: Option<String>,
    pub encoding: Option<CompressionType>
}

impl Block {
    pub fn new(index: usize, dimensions: Vector3<u32>, format: Option<usize>, data: Option<Vec<u8>>) -> Self {
        return Block {
            index,
            dimensions,
            placements: Vec::new(),
            format,
            data,
            encoding: None,
            data_url: None
        }
    }

    /// At the given offset, copy data from another block.
    /// If blocks have defines formats, these need to be the same.
    /// * `offset` - a vector representing offset inside target block where data of source block should start
    /// * `block` - a source block (required to have data!)
    /// * `format` - a format to interpret data in source block
    pub fn set_data_in_range(&mut self, offset: Vector3<u32>, block: &Block, format: &Format) -> Result<(), BlockError> {       
        let start = offset;
        let end = offset + block.dimensions;
        let extent = end - start;
        if self.data.is_none() {
            return Err(BlockError::NoData(self.index));
        }
        if block.data.is_none() {
            return Err(BlockError::NoData(block.index));
        }
        if self.format != block.format {
            return Err(BlockError::FormatMismatch(self.index, block.index));
        }
        if extent.is_any_lt(Vector3::from_xyz(0, 0, 0)) {
            return Err(BlockError::StartGreaterThanEnd(self.index, start, end));
        }
        if start.is_any_lt(Vector3::from_xyz(0, 0, 0)) {
            return Err(BlockError::StartOutOfBounds(self.index, start));
        }
        if end.is_any_gt(self.dimensions) {
            return Err(BlockError::EndOutOfBounds(self.index, end));
        }

        let microblock_dimensions = format.microblock_dimensions;
        if start.is_any_div(&microblock_dimensions) {
            return Err(BlockError::BlockInvalidPosition(self.index, start, microblock_dimensions));
        }
        if extent.is_any_div(&microblock_dimensions) {
            return Err(BlockError::BlockInvalidSize(self.index, extent, microblock_dimensions));
        }

        let microblock_size = format.microblock_size;
        let microblock_start = (start / microblock_dimensions).to_u32();
        let microblock_amount_in_range = (extent / microblock_dimensions).to_u32();
        let microblock_amount_in_block = (self.dimensions / microblock_dimensions).to_u32();

        let src_bytes;
        let src_original_len = format.count_space(block.dimensions) as usize;
        let data = block.data.as_ref().unwrap();
        let compression_scheme = &block.encoding;
        if block.encoding.is_none() {
            src_bytes = data.to_vec();
        } else {
            src_bytes = compression_scheme.as_ref().unwrap().decompress(data, src_original_len);
        }

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

    /// Copy a portion of data from self to a new block and return it.
    /// * `start` - position of source block (self) where the copy operation should start
    /// * `end` - position of source block (self) where copy operation should end
    /// * `format` - a format to interpret data in source block
    pub fn get_data_in_range(&self, start: Vector3<u32>, end: Vector3<u32>, format: &Format) -> Result<Block, BlockError> {
        let extent = end - start;
        if self.data.is_none() {
            return Err(BlockError::NoData(self.index));
        }
        if extent.is_any_lt(Vector3::from_xyz(0, 0, 0)) {
            return Err(BlockError::StartGreaterThanEnd(self.index, start, end));
        }
        if start.is_any_lt(Vector3::from_xyz(0, 0, 0)) {
            return Err(BlockError::StartOutOfBounds(self.index, start));
        }
        if end.is_any_gt(self.dimensions) {
            return Err(BlockError::EndOutOfBounds(self.index, start));
        }

        let microblock_dimensions = format.microblock_dimensions;
        if start.is_any_div(&microblock_dimensions) {
            return Err(BlockError::BlockInvalidPosition(self.index, start, microblock_dimensions));
        }
        if extent.is_any_div(&microblock_dimensions) {
            return Err(BlockError::BlockInvalidSize(self.index, start, microblock_dimensions));
        }

        let microblock_size = format.microblock_size;
        let microblock_start = (start / microblock_dimensions).to_u32();
        let microblock_amount_in_range = (extent / microblock_dimensions).to_u32();
        let microblock_amount_in_block = (self.dimensions / microblock_dimensions).to_u32();

        let mut block = Block::new(0, extent, self.format, None);
        let src_bytes = &self.data.as_ref().unwrap();
        let dest_vec_size = format.count_space(extent) as usize;
        let mut dest_bytes = Vec::with_capacity(dest_vec_size);
        // God, I sure hope dest_bytes and src_bytes are of the same size...
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

    /// Converts self to JSON object and returns JsonValue.
    pub fn to_json(&self) -> JsonValue {
        let mut hm = HashMap::new();
        let mut placements = Vec::new();
        for placement in &self.placements {
            placements.push(placement.to_json());
        }
        hm.insert("placements".to_string(), placements.into());
        hm.insert("dimensions".to_string(), self.dimensions.to_json());
        if self.format.is_some() {
            hm.insert("format".to_string(), (self.format.unwrap() as f64).into());
        }
        if self.data_url.is_some() {
            hm.insert("data".to_string(), self.data_url.as_ref().unwrap().clone().into());
        }
        if self.encoding.is_some() {
            hm.insert("encoding".to_string(), self.encoding.as_ref().unwrap().to_string().into());
        }

        return hm.into();
    }

    /// Creates a block out of JSON object and returns it.
    /// * `index` - index of the block inside BVP manifest file
    /// * `j` - JSON value, should be an object
    /// * `files` - vector of volume files to pull data from
    pub fn from_json(index: usize, j: &JsonValue, files: &Vec<File>) -> Result<Self, BlockError> {
        // All of this is probably not optimal...
        match j {
            JsonValue::Object(o) => {
                let dimensions = Vector3::<u32>::from_json(&o["dimensions"]).map_err(|x| BlockError::InvalidJson(index, x))?;
                let mut placements = Vec::new();
                match &o["placements"] {
                    JsonValue::Array(a) => {
                        for el in a {
                            let placement = match Placement::from_json(index, &el) {
                                Ok(p) => p,
                                Err(e) => return Err(BlockError::InvalidPlacement(index, e))
                            };
                            placements.push(placement);
                        }
                    },
                    _ => {
                        return Err(BlockError::InvalidJson(index, JsonError::NotAnArray(o["placements"].clone())));
                    }
                };
                let mut block = Block {
                    index,
                    dimensions,
                    placements,
                    format: None,
                    data: None,
                    data_url: None,
                    encoding: None
                };

                match o.get("format") {
                    Some(f) => {
                        let format = get_u32_from_json(f);
                        if format.is_err() {
                            return Err(BlockError::InvalidJson(index, format.unwrap_err()));
                        }
                        block.format = Some(format.unwrap() as usize);
                    },
                    None => ()
                }

                match o.get("data") {
                    Some(d) => {
                        let data_url = match get_string_from_json(d) {
                            Ok(d) => d,
                            Err(e) => return Err(BlockError::InvalidJson(index, e))
                        };
                        let encoding = match get_string_from_json(&o["encoding"]) {
                            Ok(e) => match CompressionType::from_string(&e) {
                                Ok(e) => e,
                                Err(e) => return Err(BlockError::InvalidCompression(index, e)),
                            },
                            Err(e) => return Err(BlockError::InvalidJson(index, e))
                        };
                        
                        for file in files {
                            if file.name == data_url {
                                let data = file.data.to_vec();
                                block.data_url = Some(data_url);
                                block.encoding = Some(encoding);
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
        return Err(BlockError::InvalidJson(index, JsonError::NotAnObject(j.clone())));
    }

    /// Check if data in two blocks is the same.
    /// * `vec` - bytes of data in the other block
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
}
