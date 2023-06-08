use std::{collections::HashMap, rc::Rc, fs, path::Path, str::FromStr};

use tinyjson::{JsonValue};

use crate::{block::Block, formats::Format, vector3::Vector3, json_aux::{get_string_from_json, get_u32_from_json}};

pub struct Asset {
    version: String,
    pub name: Option<String>,
    pub generator: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub copyright: Option<String>,
    pub acquisition_time: Option<String>,
    pub creation_time: Option<String>,
    pub extensions_used: Option<Vec<String>>,
    pub extensions_required: Option<Vec<String>>
}

impl Asset {
    pub fn to_hashmap(&self) -> HashMap<String, JsonValue> {
        let mut hm = HashMap::new();
        hm.insert("version".to_string(), self.version.clone().into());
        if self.name.is_some() {
            hm.insert("name".to_string(), self.name.as_ref().unwrap().clone().into());
        }
        if self.generator.is_some() {
            hm.insert("generator".to_string(), self.generator.as_ref().unwrap().clone().into());
        }
        if self.author.is_some() {
            hm.insert("author".to_string(), self.author.as_ref().unwrap().clone().into());
        }
        if self.description.is_some() {
            hm.insert("description".to_string(), self.description.as_ref().unwrap().clone().into());
        }
        if self.copyright.is_some() {
            hm.insert("copyright".to_string(), self.copyright.as_ref().unwrap().clone().into());
        }
        if self.acquisition_time.is_some() {
            hm.insert("acquisitionTime".to_string(), self.acquisition_time.as_ref().unwrap().clone().into());
        }
        if self.creation_time.is_some() {
            hm.insert("creationTime".to_string(), self.creation_time.as_ref().unwrap().clone().into());
        }
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
        let name = match hashmap.get("name") {
            Some(s) => Some(get_string_from_json(s)?),
            None => None
        };
        let generator = match hashmap.get("generator") {
            Some(s) => Some(get_string_from_json(s)?),
            None => None
        };
        let author = match hashmap.get("author") {
            Some(s) => Some(get_string_from_json(s)?),
            None => None
        };
        let description = match hashmap.get("description") {
            Some(s) => Some(get_string_from_json(s)?),
            None => None
        };
        let copyright = match hashmap.get("copyright") {
            Some(s) => Some(get_string_from_json(s)?),
            None => None
        };
        let acquisition_time = match hashmap.get("acquisitionTime") {
            Some(s) => Some(get_string_from_json(s)?),
            None => None
        };
        let creation_time = match hashmap.get("creationTime") {
            Some(s) => Some(get_string_from_json(s)?),
            None => None
        };
        let asset = Asset {
            version, name, generator, author, description, copyright, acquisition_time,
            creation_time, extensions_required: None, extensions_used: None
        };
        return Ok(asset);
    }
}

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
        hm.insert("volumeSize".to_string(), self.volume_size.to_vec().into());
        if self.voxel_size.is_some() {
            hm.insert("voxelSize".to_string(), self.voxel_size.unwrap().to_vec().into());
        }
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

        let block = get_u32_from_json(&hashmap["block"])? as usize;
        let name = match hashmap.get("name") {
            Some(s) => Some(get_string_from_json(s)?),
            None => None
        };
        let description = match hashmap.get("description") {
            Some(s) => Some(get_string_from_json(s)?),
            None => None
        };
        let semantic_type = match hashmap.get("semanticType") {
            Some(s) => Some(get_string_from_json(s)?),
            None => None
        };
        let volume_size = match hashmap.get("volumeSize") {
            Some(s) => Vector3::<f32>::from_json(s)?,
            None => Vector3::<f32>{x: 0.0, y: 0.0, z: 0.0}
        };
        let voxel_size = match hashmap.get("voxelSize") {
            Some(s) => Some(Vector3::<f32>::from_json(s)?),
            None => None
        };

        return Ok(Self::new(name, description, semantic_type, volume_size, voxel_size, block));
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

    pub fn write(&self) -> Result<(), String> {
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
        let asset = Asset {
            version: "1.0".to_string(),
            name: None,
            generator: None,
            author: None,
            description: None,
            copyright: None,
            acquisition_time: None,
            creation_time: None,
            extensions_required: None,
            extensions_used: None
        };
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