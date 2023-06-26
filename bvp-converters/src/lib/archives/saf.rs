use std::{collections::HashMap, io::{Read}, str::FromStr};
use std::sync::Arc;
use tinyjson::JsonValue;

use crate::{file::File, errors::SafError};
use crate::json_aux;

const SAF_IDENTIFIER_LENGTH: usize = 12;
const SAF_IDENTIFIER: [u8; 12] = [0xab, 0x53, 0x41, 0x46, 0x20, 0x31, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a];

/// Checks if provided data has a valid SAF identifier.
/// * `data` - raw bytes as vector of u8
pub fn check_identifier(data: &Vec<u8>) -> Result<(), SafError> {
    if data.len() < SAF_IDENTIFIER_LENGTH {
        return Err(SafError::BrokenFile);
    }
    for i in 0..SAF_IDENTIFIER_LENGTH {
        if SAF_IDENTIFIER[i] != data[i] {
            return Err(SafError::NotValidIdentifier);
        }
    }
    return Ok(());
}

/// Returns SAF manifest size.
/// * `vec` - SAF file as bytes array
/// * `offset` - index of the byte at which the manifest size starts
pub fn get_manifest_size(vec: &Vec<u8>, offset: usize) -> Result<u32, SafError> {
    if vec.len() < offset + 4 {
        return Err(SafError::BrokenFile);
    }
    let mut arr = [0, 0, 0, 0];
    for i in 0..4 {
        arr[i] = vec[offset + i];
    }
    return Ok(u32::from_le_bytes(arr));
}

/// Extracts SAF manifest from SAF file and returns it as a JSON object.
/// * `vec` - SAF file as bytes array
/// * `offset` - index of the byte at which the manifest starts
/// * `length` - the length of manifest in bytes
pub fn get_manifest(vec: &Vec<u8>, offset: usize, length: usize) -> Result<JsonValue, SafError> {
    let mut bytes = &vec[offset..offset + length];
    let mut text = String::new();
    bytes.read_to_string(&mut text).unwrap();

    match JsonValue::from_str(&text) {
        Ok(j) => {
            return Ok(j);
        },
        Err(e) => {
            return Err(SafError::ManifestCorrupt(e.to_string()));
        }
    };
}

/// Creates a SAF archive from provided file data. Returns bytes for direct writing.
/// * `files` - a vector of files to write
pub fn to_saf_archive(files: &Vec<File>) -> Result<Vec<u8>, SafError> {
    let mut manifest = Vec::new();
    for file in files {
        let mut file_hashmap = HashMap::new();
        file_hashmap.insert("path".to_string(), file.name.clone().into());
        if file.mime.is_some() {
            file_hashmap.insert("mime".to_string(), file.mime.as_ref().unwrap().clone().into());
        }
        file_hashmap.insert("size".to_string(), (file.data.len() as f64).into());
        manifest.push(file_hashmap.into());
    }
    let json = JsonValue::from(manifest);
    let text = match json.stringify() {
        Ok(t) => t,
        Err(e) => {
            return Err(SafError::ManifestCorrupt(e.to_string()));
        },
    };
    let manifest_buffer = text.as_bytes();
    let manifest_size = manifest_buffer.len();
    let manifest_size_buffer = (manifest_size as u32).to_le_bytes();
    let saf_size = SAF_IDENTIFIER_LENGTH + manifest_size_buffer.len() + manifest_buffer.len() + files.iter().map(|e| e.data.len()).sum::<usize>();
    let mut saf: Vec<u8> = Vec::with_capacity(saf_size);
    for i in SAF_IDENTIFIER {
        saf.push(i);
    }
    for i in manifest_size_buffer {
        saf.push(i);
    }
    for i in manifest_buffer {
        saf.push(*i);
    }
    for file in files {
        for i in file.data.as_ref() {
            saf.push(*i);
        }
    }

    return Ok(saf);
}

/// Extracts files from SAF archive. Returns a vectors of raw files.
/// * `saf` - SAF file as bytes
pub fn from_saf_archive(saf: &Vec<u8>) -> Result<Vec<File>, SafError> {
    check_identifier(saf)?;
    
    let mut offset = SAF_IDENTIFIER_LENGTH;
    let manifest_size = get_manifest_size(saf, offset)? as usize;
    offset += 4;
    let manifest = get_manifest(saf, offset, manifest_size)?;
    offset += manifest_size;
    let manifest_files = match json_aux::get_array_from_json(&manifest) {
        Ok(m) => m,
        Err(e) => return Err(SafError::InvalidJson(e))
    };

    let mut files = Vec::new();

    for file_entry in manifest_files {
        match file_entry {
            JsonValue::Object(o) => {
                let path = match json_aux::get_string_from_json(&o["path"]) {
                    Ok(p) => p,
                    Err(e) => return Err(SafError::InvalidJson(e))
                };
                let mime = match o.get("mime") {
                    Some(s) => match json_aux::get_string_from_json(s) {
                        Ok(m) => m,
                        Err(e) => return Err(SafError::InvalidJson(e))
                    },
                    None => String::new()
                };
                let size = match json_aux::get_u32_from_json(&o["size"]) {
                    Ok(s) => s as usize,
                    Err(e) => return Err(SafError::InvalidJson(e))
                };
                let data = saf[offset..offset+size].to_vec();
                let file = File::new(path, Arc::new(data), Some(mime));
                files.push(file);
                offset += size;
            },
            _ => ()
        }
    }

    return Ok(files);
}