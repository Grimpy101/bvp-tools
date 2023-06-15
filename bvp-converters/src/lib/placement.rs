use std::collections::HashMap;

use tinyjson::JsonValue;

use crate::{vector3::Vector3, json_aux::get_u32_from_json, errors::{PlacementError, JsonError}};

pub struct Placement {
    pub position: Vector3<u32>,
    pub block: usize
}

impl Placement {
    pub fn new(position: Vector3<u32>, block: usize) -> Self {
        return Self { position, block };
    }

    pub fn to_json(&self) -> JsonValue {
        let mut hm = HashMap::new();
        hm.insert("position".to_string(), self.position.to_json());
        hm.insert("block".to_string(), (self.block as f64).into());
        return hm.into();
    }

    pub fn from_json(block_index: usize, j: &JsonValue) -> Result<Self, PlacementError> {
        return match j {
            JsonValue::Object(o) => {
                let position = match Vector3::<u32>::from_json(&o["position"]) {
                    Ok(p) => p,
                    Err(e) => return Err(PlacementError::InvalidJson(block_index, e))
                };
                let block = match get_u32_from_json(&o["block"]) {
                    Ok(b) => b,
                    Err(e) => return Err(PlacementError::InvalidJson(block_index, e))
                };
                return Ok(Placement::new(position, block as usize));
            },
            _ => Err(PlacementError::InvalidJson(block_index, JsonError::NotAnObject(j.clone())))
        };
    }
}