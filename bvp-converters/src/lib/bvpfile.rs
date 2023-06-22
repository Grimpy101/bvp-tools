use std::{collections::{HashMap, HashSet}, str::FromStr};

use tinyjson::{JsonValue};

use crate::{block::Block, formats::Format, asset::Asset, modality::Modality, file::File, errors::{BvpFileError, JsonError}};


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
            extensions_required: Vec::new(),
            extensions_used: Vec::new()
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
        let mut formats = Vec::new();
        let mut modalities = Vec::new();
        let mut blocks = Vec::new();
        let mut extensions = HashSet::new();

        for format in &self.formats {
            formats.push(format.to_json().into());
            if format.extension.is_some() {
                extensions.insert(format.extension.unwrap());
            }
        }
        for modality in &self.modalities {
            modalities.push(modality.to_json());
        }
        for block in &self.blocks {
            blocks.push(block.to_json());
        }

        let asset = self.asset.to_json(extensions);
        let mut manifest = HashMap::new();
        manifest.insert("asset".to_string(), asset);
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

    pub fn from_manifest(manifest_content: &str, files: &Vec<File>) -> Result<Self, BvpFileError> {
        let mut state = BVPFile::new();
        let json = match JsonValue::from_str(manifest_content) {
            Ok(j) => match j {
                JsonValue::Object(o) => o,
                _ => {
                    return Err(BvpFileError::InvalidJson(JsonError::NotAnObject(j.clone())));
                }
            },
            Err(e) => {
                return Err(BvpFileError::BrokenManifest(e.to_string()));
            },
        };

        state.asset = match Asset::from_json(&json["asset"]) {
            Ok(a) => a,
            Err(e) => return Err(BvpFileError::AssetError(e))
        };

        match &json["blocks"] {
            JsonValue::Array(a) => {
                for (i, el) in a.iter().enumerate() {
                    let block = match Block::from_json(i, &el, files) {
                        Ok(b) => b,
                        Err(e) => return Err(BvpFileError::BlockError(e))
                    };
                    state.blocks.push(block);
                }
            },
            _ => {
                return Err(BvpFileError::InvalidJson(JsonError::NotAnArray(json["block"].clone())));
            }
        };
        match &json["modalities"] {
            JsonValue::Array(a) => {
                for (i, el) in a.iter().enumerate() {
                    let modality = match Modality::from_json(i, &el) {
                        Ok(m) => m,
                        Err(e) => return Err(BvpFileError::ModalityError(e))
                    };
                    state.modalities.push(modality);
                }
            },
            _ => {
                return Err(BvpFileError::InvalidJson(JsonError::NotAnArray(json["modalities"].clone())));
            }
        };
        match &json["formats"] {
            JsonValue::Array(a) => {
                for el in a {
                    let format = match Format::from_json(&el) {
                        Ok(f) => f,
                        Err(e) => return Err(BvpFileError::FormatError(e))
                    };
                    state.formats.push(format);
                }
            },
            _ => {
                return Err(BvpFileError::InvalidJson(JsonError::NotAnArray(json["formats"].clone())));
            }
        };

        return Ok(state);
    }
}