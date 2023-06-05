use std::{collections::HashMap, rc::Rc, fs, path::Path, str::FromStr};

use tinyjson::{JsonValue};

use crate::{block::Block, formats::Format, vector3::Vector3, json_aux::{get_string_from_json, get_u32_from_json}};

pub struct Asset {
    version: String
}

impl Asset {
    pub fn to_hashmap(&self) -> HashMap<String, JsonValue> {
        let mut hm = HashMap::new();
        hm.insert("version".to_string(), self.version.clone().into());
        return hm;
    }

    pub fn from_json(j: &JsonValue) -> Result<Self, String> {
        let hashmap = match j {
            JsonValue::Object(o) => o,
            _ => {
                return Err("Invalid JSON".to_string());
            }
        };
        let version = get_string_from_json(&hashmap["version"])?;
        let asset = Asset { version };
        return Ok(asset);
    }
}

pub struct Modality {
    pub name: Option<String>,
    description: Option<String>,
    semantic_type: Option<String>,
    scale: Vector3<f32>,
    pub block: usize
}

impl Modality {
    pub fn new(name: Option<String>, description: Option<String>, semantic_type: Option<String>, scale: Vector3<f32>, block: usize) -> Self {
        return Self { name, description, semantic_type, scale, block };
    }

    pub fn to_hashmap(&self) -> HashMap<String, JsonValue> {
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
        hm.insert("scale".to_string(), self.scale.to_vec().into());
        hm.insert("block".to_string(), (self.block as f64).into());
        return hm;
    }

    pub fn from_json(j: &JsonValue) -> Result<Self, String> {
        let hashmap = match j {
            JsonValue::Object(o) => o,
            _ => {
                return Err("Invalid JSON".to_string());
            }
        };

        let scale = Vector3::<f32>::from_json(&hashmap["scale"])?;
        let block = get_u32_from_json(&hashmap["block"])? as usize;

        return Ok(Self::new(None, None, None, scale, block));
    }
}

pub struct File {
    pub name: String,
    pub data: Rc<Vec<u8>>,
    pub mime: Option<String>
}

impl File {
    pub fn new(name: String, data: Rc<Vec<u8>>, mime: Option<String>) -> Self {
        return Self { name, data, mime };
    }

    pub fn _write(&self) -> Result<(), String> {
        let path = Path::new(&self.name);
        let prefix = path.parent().unwrap();
        match fs::create_dir_all(prefix) {
            Ok(_) => (),
            Err(_) => {
                return Err(format!("Could not create path {:?}", prefix));
            },
        };
        match fs::write(&self.name, self.data.as_slice()) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!("Error writing file {}: {}", self.name, e));
            },
        };

        return Ok(());
    }
}

pub struct BVPFile {
    pub asset: Asset,
    pub modalities: Vec<Modality>,
    pub blocks: Vec<Block>,
    pub formats: Vec<Format>,
    pub block_map: HashMap<u64, usize>,
    pub files: Vec<File>
}

impl BVPFile {
    pub fn new() -> Self {
        let asset = Asset { version: "1.0".to_string() };
        let modalities = Vec::new();
        let blocks = Vec::new();
        let formats = Vec::new();
        let block_map = HashMap::new();
        let files = Vec::new();
        return Self {
            asset,
            modalities,
            blocks,
            formats,
            block_map,
            files
        }
    }

    pub fn to_manifest(&self) -> Result<Vec<u8>, String> {
        let asset = self.asset.to_hashmap();
        let mut formats = Vec::new();
        let mut modalities = Vec::new();
        let mut blocks = Vec::new();

        for format in &self.formats {
            formats.push(format.to_json().into());
        }
        for modality in &self.modalities {
            modalities.push(modality.to_hashmap().into());
        }
        for block in &self.blocks {
            blocks.push(block.to_hashmap().into());
        }

        let mut manifest = HashMap::new();
        manifest.insert("asset".to_string(), asset.into());
        manifest.insert("formats".to_string(), formats.into());
        manifest.insert("modalities".to_string(), modalities.into());
        manifest.insert("blocks".to_string(), blocks.into());

        let v = JsonValue::from(manifest);
        let content = match v.stringify() {
            Ok(c) => c,
            Err(e) => {
                return Err(format!("Error creating manifest JSON: {}", e));
            },
        };
        return Ok(content.into_bytes());
    }

    pub fn from_manifest(manifest_content: &str, files: &Vec<File>) -> Result<Self, String> {
        let mut state = BVPFile::new();
        let json = match JsonValue::from_str(manifest_content) {
            Ok(j) => match j {
                JsonValue::Object(o) => o,
                _ => {
                    return Err("Invalid JSON in manifest".to_string());
                }
            },
            Err(_) => {
                return Err("Invalid JSON in manifest".to_string());
            },
        };

        state.asset = Asset::from_json(&json["asset"])?;

        match &json["blocks"] {
            JsonValue::Array(a) => {
                for el in a {
                    let block = Block::from_json(&el, files)?;
                    state.blocks.push(block);
                }
            },
            _ => {
                return Err("Invalid JSON in manifest".to_string());
            }
        };
        match &json["modalities"] {
            JsonValue::Array(a) => {
                for el in a {
                    let modality = Modality::from_json(&el)?;
                    state.modalities.push(modality);
                }
            },
            _ => {
                return Err("Invalid JSON in manifest".to_string());
            }
        };
        match &json["formats"] {
            JsonValue::Array(a) => {
                for el in a {
                    let format = Format::from_json(&el)?;
                    state.formats.push(format);
                }
            },
            _ => {
                return Err("Invalid JSON in manifest".to_string());
            }
        };

        return Ok(state);
    }
}