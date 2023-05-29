use std::{collections::HashMap, io::Write, path};

use tinyjson::JsonValue;

use crate::bvpfile::File;

const SAF_SIGNATURE: [u8; 12] = [0xab, 0x53, 0x41, 0x46, 0x20, 0x31, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a];

pub fn fill_bytes_from_usize(mut bytes: &mut [u8], n: u32) -> Result<(), String> {
    return match bytes.write(&n.to_le_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => {
            Err(format!("Could not fill bytes: {}", e))
        }
    };
}

/*pub fn to_saf_archive(files: &Vec<File>) -> Result<Vec<u8>, String> {
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
        println!("Archiving file {}...", file.name);
        for i in file.data.as_ref() {
            saf.push(*i);
        }
    }
    println!("Man. size: {}, Saf size: {}", manifest_size, saf_size);

    return Ok(saf);
}*/

pub fn to_saf_archive(files: &Vec<File>) -> Result<Vec<u8>, String> {
    let mut saf: Vec<u8> = "SAF1\n".bytes().collect();

    let mut folders = Vec::new();

    for file in files {
        let path = path::Path::new(&file.name);
        let prefix = path.parent().unwrap();
        if !folders.contains(&prefix.to_str().unwrap().to_string()) {
            let mut folder_header_hm = HashMap::new();
            folder_header_hm.insert("Name".to_string(), prefix.to_str().unwrap().to_string().clone().into());
            folder_header_hm.insert("Type".to_string(), 1f64.into());
            let json = JsonValue::from(folder_header_hm);
            let content = json.stringify().unwrap();
            let mut content_bytes: Vec<u8> = content.bytes().collect();
            let mut content_size: Vec<u8> = format!("{}\n", content_bytes.len()).bytes().collect();
            saf.append(&mut content_size);
            saf.append(&mut content_bytes);

            folders.push(prefix.to_str().unwrap().to_string());
        }

        let mut file_header_hm = HashMap::new();
        file_header_hm.insert("Name".to_string(), file.name.clone().into());
        file_header_hm.insert("Length".to_string(), (file.data.len() as f64).into());
        file_header_hm.insert("Type".to_string(), 0f64.into());
        let json = JsonValue::from(file_header_hm);
        let content = json.stringify().unwrap();
        let mut content_bytes: Vec<u8> = content.bytes().collect();
        let mut content_size: Vec<u8> = format!("{}\n", content_bytes.len()).bytes().collect();
        saf.append(&mut content_size);
        saf.append(&mut content_bytes);

        for i in file.data.as_ref() {
            saf.push(*i);
        }
    }

    return Ok(saf);
}