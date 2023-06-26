use std::collections::HashMap;

use tinyjson::JsonValue;

use crate::{vector3::Vector3, errors::{ModalityError, JsonError}, json_aux};

#[derive(Debug)]
pub struct Modality {
    pub name: Option<String>,
    description: Option<String>,
    semantic_type: Option<String>,
    volume_size: Vector3<f32>,
    voxel_size: Option<Vector3<f32>>,
    pub block: usize
}

impl Modality {
    pub fn new(name: Option<String>, description: Option<String>, semantic_type: Option<String>,
        volume_size: Vector3<f32>, voxel_size: Option<Vector3<f32>>, block: usize) -> Self {
        return Self { name, description, semantic_type, volume_size, voxel_size, block };
    }

    pub fn to_json(&self) -> JsonValue {
        let mut hm = HashMap::new();
        if self.name.is_some() {
            hm.insert("name".to_string(), self.name.as_ref().unwrap().clone().into());
        }
        if self.description.is_some() {
            hm.insert("description".to_string(), self.description.as_ref().unwrap().clone().into());
        }
        if self.semantic_type.is_some() {
            hm.insert("semanticType".to_string(), self.semantic_type.as_ref().unwrap().clone().into());
        }
        hm.insert("volumeSize".to_string(), self.volume_size.to_json());
        if self.voxel_size.is_some() {
            hm.insert("voxelSize".to_string(), self.voxel_size.unwrap().to_json());
        }
        hm.insert("block".to_string(), (self.block as f64).into());
        return hm.into();
    }

    pub fn from_json(index: usize, j: &JsonValue) -> Result<Self, ModalityError> {
        let hashmap = match j {
            JsonValue::Object(o) => o,
            _ => return Err(ModalityError::InvalidJson(index, JsonError::NotAnObject(j.clone())))
        };

        let block = match json_aux::get_u32_from_json(&hashmap["block"]) {
            Ok(b) => b as usize,
            Err(e) => return Err(ModalityError::InvalidJson(index, e))
        };
        let name = match hashmap.get("name") {
            Some(s) => {
                match json_aux::get_string_from_json(s) {
                    Ok(n) => Some(n),
                    Err(e) => return Err(ModalityError::InvalidJson(index, e))
                }
            },
            None => None
        };
        let description = match hashmap.get("description") {
            Some(s) => {
                match json_aux::get_string_from_json(s) {
                    Ok(n) => Some(n),
                    Err(e) => return Err(ModalityError::InvalidJson(index, e))
                }
            },
            None => None
        };
        let semantic_type = match hashmap.get("semanticType") {
            Some(s) => {
                match json_aux::get_string_from_json(s) {
                    Ok(n) => Some(n),
                    Err(e) => return Err(ModalityError::InvalidJson(index, e))
                }
            },
            None => None
        };
        let volume_size = match hashmap.get("volumeSize") {
            Some(s) => match Vector3::<f32>::from_json(s) {
                Ok(v) => v,
                Err(e) => return Err(ModalityError::InvalidJson(index, e))
            },
            None => Vector3::<f32>{x: 0.0, y: 0.0, z: 0.0}
        };
        let voxel_size = match hashmap.get("voxelSize") {
            Some(s) => {
                match Vector3::<f32>::from_json(s) {
                    Ok(v) => Some(v),
                    Err(e) => return Err(ModalityError::InvalidJson(index, e))
                }
            },
            None => None
        };

        return Ok(Self::new(name, description, semantic_type, volume_size, voxel_size, block));
    }
}