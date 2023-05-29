use std::collections::HashMap;

use tinyjson::JsonValue;

use crate::{vector3::Vector3};

pub struct Placement {
    pub position: Vector3<u32>,
    pub block: usize
}

impl Placement {
    pub fn new(position: Vector3<u32>, block: usize) -> Self {
        return Self { position, block };
    }

    pub fn to_hashmap(&self) -> HashMap<String, JsonValue> {
        let mut hm = HashMap::new();
        hm.insert("position".to_string(), self.position.to_f64_vec().into());
        hm.insert("block".to_string(), (self.block as f64).into());
        return hm;
    }
}