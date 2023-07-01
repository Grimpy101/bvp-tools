use std::{path::Path, fs, str::FromStr, collections::HashMap};
use std::sync::Arc;

use tinyjson::JsonValue;

use crate::{errors::{ArchiveError}, file::File};

use super::ArchiveWriter;

pub struct RawFilesWriter {

}

impl RawFilesWriter {
    pub fn new() -> Self {
        return Self {  };
    }
}

impl ArchiveWriter for RawFilesWriter {
    fn append_file(&mut self, file: &File) -> Result<(), String> {
        let path = Path::new(&file.name);
        if path.parent().is_some() {
            let parent = path.parent().unwrap();
            if !parent.try_exists().expect("Error: Cannot determine if folder exists") {
                fs::create_dir_all(parent).map_err(|err| err.to_string())?;
            } else if parent.is_file() {
                return Err("Destination folder is a file!".to_string());
            }
        }

        fs::write(path, file.data.as_slice()).map_err(|err| err.to_string())?;
        return Ok(());
    }

    fn finish(&self, _path: String) -> Result<(), String> {
        return Ok(());
    }
}

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
            let file = File::new(data_path, Arc::new(data_content), None);
            files.push(file);
        }
    }

    let manifest_file = File::new(filepath.to_string_lossy().to_string(), Arc::new(manifest_contents), Some("application/json".to_string()));
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