use std::{path::Path, fs, str::FromStr, collections::HashMap, rc::Rc};

use tinyjson::JsonValue;

use crate::{errors::{ArchiveError}, file::File};

pub fn from_manifest_file(filepath: &Path) -> Result<Vec<File>, ArchiveError> {
    let mut files = Vec::new();

    let manifest_contents = match fs::read(filepath) {
        Ok(v) => v,
        Err(e) => return Err(ArchiveError::CannotRead(e.to_string()))
    };

    let content = match std::str::from_utf8(&manifest_contents) {
        Ok(c) => c,
        Err(e) => {
            return Err(ArchiveError::NotValidFile(format!("Cannot decode manifest: ({})", e)));
        },
    };

    let json = JsonValue::from_str(&content).map_err(|x| ArchiveError::NotValidFile(format!("Invalid JSON ({})", x)))?;
    let hash_map: HashMap<String, JsonValue> = json.try_into().map_err(|_| ArchiveError::NotValidFile("Invalid JSON".to_string()))?;
    let blocks: Vec<JsonValue> = hash_map["blocks"].clone().try_into().map_err(|_| ArchiveError::NotValidFile("Invalid JSON".to_string()))?;
    for block in blocks {
        let block: HashMap<String, JsonValue> = block.try_into().map_err(|_| ArchiveError::NotValidFile("Invalid JSON".to_string()))?;
        if block.get("data").is_some() {
            let data_path: String = block["data"].clone().try_into().map_err(|_| ArchiveError::NotValidFile("Invalid JSON".to_string()))?;
            let full_data_path = Path::new(filepath.parent().unwrap()).join(&data_path);
            let data_content = fs::read(&full_data_path).map_err(|x| ArchiveError::CannotRead(format!("Could not read file {} ({})", full_data_path.display(), x)))?;
            let file = File::new(data_path, Rc::new(data_content), None);
            files.push(file);
        }
    }

    let manifest_file = File::new(filepath.to_string_lossy().to_string(), Rc::new(manifest_contents), Some("application/json".to_string()));
    files.push(manifest_file);

    return Ok(files);
}

pub fn from_folder(filepath: &Path) -> Result<Vec<File>, ArchiveError> {
    let rd = fs::read_dir(filepath).map_err(|x| ArchiveError::CannotRead(x.to_string()))?;

    for entry in rd {
        let entry = entry.map_err(|x| ArchiveError::CannotRead(x.to_string()))?;
        if entry.file_name() == "manifest.json" {
            let path = entry.path();
            let manifest_file = path.as_path();
            return from_manifest_file(manifest_file);
        }
    }

    return Err(ArchiveError::NotValidFile("No manifest file".to_string()))
}