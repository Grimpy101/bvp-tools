use std::collections::{HashMap, HashSet};

use tinyjson::JsonValue;

use crate::{errors::{AssetError, JsonError}, json_aux, extensions::Extension};

#[derive(Debug)]
pub struct Asset {
    pub version: String,
    pub name: Option<String>,
    pub generator: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub copyright: Option<String>,
    pub acquisition_time: Option<String>,
    pub creation_time: Option<String>,
    pub extensions_used: Vec<String>,
    pub extensions_required: Vec<String>
}

impl Asset {
    pub fn to_json(&self, ext: HashSet<Extension>) -> JsonValue {
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
        if ext.len() > 0 {
            let mut ext_used: Vec<JsonValue> = Vec::new();
            let mut ext_req: Vec<JsonValue> = Vec::new();
            for e in &ext {
                ext_used.push(e.to_string().into());
                ext_req.push(e.to_string().into());
            }
            hm.insert("extensionsUsed".to_string(), ext_used.into());
            hm.insert("extensionsRequired".to_string(), ext_req.into());
        }
        return hm.into();
    }

    pub fn from_json(j: &JsonValue) -> Result<Self, AssetError> {
        let hashmap = match j {
            JsonValue::Object(o) => o,
            _ => {
                return Err(AssetError::InvalidJson(JsonError::NotAnObject(j.clone())));
            }
        };
        let version = match json_aux::get_string_from_json(&hashmap["version"]) {
            Ok(v) => v,
            Err(e) => return Err(AssetError::InvalidJson(e))
        };
        let name = match hashmap.get("name") {
            Some(s) => {
                match json_aux::get_string_from_json(s) {
                    Ok(n) => Some(n),
                    Err(e) => return Err(AssetError::InvalidJson(e))
                }
            },
            None => None
        };
        let generator = match hashmap.get("generator") {
            Some(s) => {
                match json_aux::get_string_from_json(s) {
                    Ok(g) => Some(g),
                    Err(e) => return Err(AssetError::InvalidJson(e))
                }
            },
            None => None
        };
        let author = match hashmap.get("author") {
            Some(s) => {
                match json_aux::get_string_from_json(s) {
                    Ok(a) => Some(a),
                    Err(e) => return Err(AssetError::InvalidJson(e))
                }
            },
            None => None
        };
        let description = match hashmap.get("description") {
            Some(s) => {
                match json_aux::get_string_from_json(s) {
                    Ok(d) => Some(d),
                    Err(e) => return Err(AssetError::InvalidJson(e))
                }
            },
            None => None
        };
        let copyright = match hashmap.get("copyright") {
            Some(s) => {
                match json_aux::get_string_from_json(s) {
                    Ok(c) => Some(c),
                    Err(e) => return Err(AssetError::InvalidJson(e))
                }
            },
            None => None
        };
        let acquisition_time = match hashmap.get("acquisitionTime") {
            Some(s) => {
                match json_aux::get_string_from_json(s) {
                    Ok(a) => Some(a),
                    Err(e) => return Err(AssetError::InvalidJson(e))
                }
            },
            None => None
        };
        let creation_time = match hashmap.get("creationTime") {
            Some(s) => {
                match json_aux::get_string_from_json(s) {
                    Ok(c) => Some(c),
                    Err(e) => return Err(AssetError::InvalidJson(e))
                }
            },
            None => None
        };
        let mut extensions_required = Vec::new();
        if hashmap.get("extensionsRequired").is_some() {
            extensions_required = json_aux::get_string_vec_from_json(&hashmap["extensionsRequired"]).map_err(|x| AssetError::InvalidJson(x))?;
        }
        let mut extensions_used = Vec::new();
        if hashmap.get("extensionsUsed").is_some() {
            extensions_used = json_aux::get_string_vec_from_json(&hashmap["extensionsUsed"]).map_err(|x| AssetError::InvalidJson(x))?;
        }
        let asset = Asset {
            version, name, generator, author, description, copyright, acquisition_time,
            creation_time, extensions_required, extensions_used
        };
        return Ok(asset);
    }
}