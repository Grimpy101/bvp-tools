use std::{collections::HashMap, io::{Write, Read}, str::FromStr, rc::Rc};

use tinyjson::JsonValue;

use crate::{bvpfile::File, json_aux};

const SAF_SIGNATURE: [u8; 12] = [0xab, 0x53, 0x41, 0x46, 0x20, 0x31, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a];

pub fn fill_bytes_from_usize(mut bytes: &mut [u8], n: u32) -> Result<(), String> {
    return match bytes.write(&n.to_le_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => {
            Err(format!("Could not fill bytes: {}", e))
        }
    };
}

pub fn to_saf_archive(files: &Vec<File>) -> Result<Vec<u8>, String> {
    let saf_signature: Vec<u8> = SAF_SIGNATURE.iter().flat_map(|e| e.to_le_bytes()).collect();
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
        Err(_) => {
            return Err("Error stringifying JSON".to_string());
        },
    };
    let manifest_buffer = text.as_bytes();
    let manifest_size = manifest_buffer.len();
    let mut manifest_size_buffer = [0u8; 4];
    fill_bytes_from_usize(&mut manifest_size_buffer, manifest_size as u32)?;
    let saf_size = saf_signature.len() + manifest_size_buffer.len() + manifest_buffer.len() + files.iter().map(|e| e.data.len()).sum::<usize>();
    let mut saf: Vec<u8> = Vec::with_capacity(saf_size);
    for i in saf_signature {
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

pub fn check_signature(data: &Vec<u8>) -> i32 {
    let saf_signature: Vec<u8> = SAF_SIGNATURE.iter().flat_map(|e| e.to_le_bytes()).collect();
    if data.len() < saf_signature.len() {
        return -1;
    }
    for i in 0..saf_signature.len() {
        if saf_signature[i] != data[i] {
            return -1;
        }
    }
    return saf_signature.len() as i32;
}

pub fn get_manifest_size(vec: &Vec<u8>, offset: usize) -> Result<u32, String> {
    if vec.len() < offset + 4 {
        return Err("File not big enough".to_string());
    }
    let mut arr = [0, 0, 0, 0];
    for i in 0..4 {
        arr[i] = vec[offset + i];
    }
    return Ok(u32::from_le_bytes(arr));
}

pub fn get_manifest(vec: &Vec<u8>, offset: usize, length: usize) -> Result<JsonValue, String> {
    let mut bytes = &vec[offset..offset + length];
    let mut text = String::new();
    bytes.read_to_string(&mut text);

    match JsonValue::from_str(&text) {
        Ok(j) => {
            return Ok(j);
        },
        Err(e) => {
            return Err("Cannot open file: SAF manifest corrupt".to_string());
        }
    };
}

pub fn from_saf_archive(saf: &Vec<u8>) -> Result<Vec<File>, String> {
    let signature_offset = check_signature(saf);
    if signature_offset < 0  {
        return Err("Invalid SAF signature".to_string());
    }
    let mut offset = signature_offset as usize;
    let manifest_size = get_manifest_size(saf, offset)? as usize;
    offset += 4;
    let manifest = get_manifest(saf, offset, manifest_size)?;
    offset += manifest_size;
    let manifest_files = json_aux::get_array_from_json(&manifest)?;

    let mut files = Vec::new();

    for file_entry in manifest_files {
        match file_entry {
            JsonValue::Object(o) => {
                let path = json_aux::get_string_from_json(&o["path"])?;
                let mime = match o.get("mime") {
                    Some(s) => json_aux::get_string_from_json(s)?,
                    None => String::new()
                };
                let size = json_aux::get_u32_from_json(&o["size"])? as usize;
                let data = saf[offset..offset+size].to_vec();
                let file = File::new(path, Rc::new(data), Some(mime));
                files.push(file);
                offset += size;
            },
            _ => ()
        }
    }

    return Ok(files);
}