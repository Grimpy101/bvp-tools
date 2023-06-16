use std::rc::Rc;

use chrono::{Datelike, Timelike};

use crate::{file::File, errors::ZipError};

static LOCAL_FILE_HEADER_SIG: u32 = 0x04034b50;
static CENTRAL_DIR_FILE_HEADER_SIG: u32 = 0x02014b50;
static EOCD_SIG: u32 = 0x06054b50;

struct CentralDirectoryHeader {
    version_made: u16,
    extraction_version: u16,
    general_purpose_bit: u16,
    compression_method: u16,
    last_modified_time_date: [u16; 2],
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    disk_number: u16,
    internal_attributes: u16,
    external_attributes: u32,
    relative_offset: u32,
    filename: String,
    extra_field: String,
    comment: String
}

impl CentralDirectoryHeader {
    pub fn simple_new(file: &File, offset: u32) -> Self {
        let mod_datetime = chrono::offset::Utc::now();
        let mod_day = mod_datetime.day() as u16;
        let mod_month = mod_datetime.month() as u16;
        let mod_year = (mod_datetime.year() - 1980) as u16;
        let mod_second = (mod_datetime.second() / 2) as u16;
        let mod_minute = mod_datetime.minute() as u16;
        let mod_hour = mod_datetime.hour() as u16;

        let date = mod_day | (mod_month << 5) | (mod_year << 9);
        let time = mod_second | (mod_minute << 5) | (mod_hour << 11);

        return Self {
            version_made: 0,
            extraction_version: 0,
            general_purpose_bit: 0,
            compression_method: 0,
            last_modified_time_date: [time, date],
            crc32: compute_crc32(&file.data),
            compressed_size: file.data.len() as u32,
            uncompressed_size: file.data.len() as u32,
            disk_number: 1,
            internal_attributes: 0,
            external_attributes: 0,
            relative_offset: offset,
            filename: file.name.clone(),
            extra_field: String::new(),
            comment: String::new()
        }
    }

    pub fn file_header_bytes(&self) -> Vec<u8> {
        let filename_bytes = self.filename.as_bytes();
        let extra_bytes = self.extra_field.as_bytes();
        return [
            &LOCAL_FILE_HEADER_SIG.to_le_bytes() as &[u8],
            &self.extraction_version.to_le_bytes() as &[u8],
            &self.general_purpose_bit.to_le_bytes() as &[u8],
            &self.compression_method.to_le_bytes() as &[u8],
            &self.last_modified_time_date[0].to_le_bytes() as &[u8],
            &self.last_modified_time_date[1].to_le_bytes() as &[u8],
            &self.crc32.to_le_bytes() as &[u8],
            &self.compressed_size.to_le_bytes() as &[u8],
            &self.uncompressed_size.to_le_bytes() as &[u8],
            &(filename_bytes.len() as u16).to_le_bytes() as &[u8],
            &(extra_bytes.len() as u16).to_le_bytes() as &[u8],
            &filename_bytes,
            &extra_bytes
        ].concat();
    }

    pub fn file_entry_header_len(&self) -> u32 {
        return 30 + self.filename.as_bytes().len() as u32 + self.extra_field.as_bytes().len() as u32;
    }

    pub fn central_dir_file_header(&self) -> Vec<u8> {
        let filename_bytes = self.filename.as_bytes();
        let extra_bytes = self.extra_field.as_bytes();
        let comment_bytes = self.comment.as_bytes();
        return [
            &CENTRAL_DIR_FILE_HEADER_SIG.to_le_bytes() as &[u8],
            &self.version_made.to_le_bytes() as &[u8],
            &self.extraction_version.to_le_bytes() as &[u8],
            &self.general_purpose_bit.to_le_bytes() as &[u8],
            &self.compression_method.to_le_bytes() as &[u8],
            &self.last_modified_time_date[0].to_le_bytes() as &[u8],
            &self.last_modified_time_date[1].to_le_bytes() as &[u8],
            &self.crc32.to_le_bytes() as &[u8],
            &self.compressed_size.to_le_bytes() as &[u8],
            &self.uncompressed_size.to_le_bytes() as &[u8],
            &(filename_bytes.len() as u16).to_le_bytes() as &[u8],
            &(extra_bytes.len() as u16).to_le_bytes() as &[u8],
            &(comment_bytes.len() as u16).to_le_bytes() as &[u8],
            &self.disk_number.to_le_bytes() as &[u8],
            &self.internal_attributes.to_le_bytes() as &[u8],
            &self.external_attributes.to_le_bytes() as &[u8],
            &self.relative_offset.to_le_bytes() as &[u8],
            &filename_bytes,
            &extra_bytes,
            &comment_bytes
        ].concat();
    }
}


pub fn compute_crc32(data: &Vec<u8>) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&data);
    return hasher.finalize();
}

pub fn to_zip_archive(files: &Vec<File>) -> Result<Vec<u8>, ZipError> {  
    let zip_size = 0;
    let mut zip: Vec<u8> = Vec::with_capacity(zip_size);

    let mut central_file_headers = Vec::with_capacity(files.len());
    let mut offset = 0;
    for file in files {
        let file_header = CentralDirectoryHeader::simple_new(file, offset);
        offset += file_header.file_entry_header_len() + file.data.len() as u32;

        zip.append(&mut file_header.file_header_bytes());
        for d in file.data.iter() {
            zip.push(*d);
        }
        central_file_headers.push(file_header);
    }

    let mut central_dir_size = 0;
    let central_dir_offset = offset;
    for file_header in &central_file_headers {
        let mut central_dir_file_header = file_header.central_dir_file_header();
        central_dir_size += central_dir_file_header.len();
        zip.append(&mut central_dir_file_header);
    }

    let mut eocd = [
        &EOCD_SIG.to_le_bytes() as &[u8],
        &1u16.to_le_bytes() as &[u8],
        &1u16.to_le_bytes() as &[u8],
        &(central_file_headers.len() as u16).to_le_bytes() as &[u8],
        &(central_file_headers.len() as u16).to_le_bytes() as &[u8],
        &(central_dir_size as u32).to_le_bytes() as &[u8],
        &(central_dir_offset as u32).to_le_bytes() as &[u8],
        &0u16.to_le_bytes() as &[u8]
    ].concat();

    zip.append(&mut eocd);

    return Ok(zip);
}

pub fn find_eocd(data: &Vec<u8>) -> Result<usize, ZipError> {
    let mut i = (data.len() as i64) - 1;
    while i - 3 >= 0 {
        let b1 = data[(i - 3) as usize] as u32;
        let b2 = data[(i - 2) as usize] as u32;
        let b3 = data[(i - 1) as usize] as u32;
        let b4 = data[i as usize] as u32;
        let code = b4 << 24 | b3 << 16 | b2 << 8 | b1;
        if code == EOCD_SIG {
            return Ok((i + 1) as usize);
        }
        i = i - 1;
    }
    return Err(ZipError::CorruptFile("EOCD not found".to_string()));
}

pub fn get_u32_from_data(data: &Vec<u8>, offset: usize) -> u32 {
    let b1 = data[offset] as u32;
    let b2 = data[offset + 1] as u32;
    let b3 = data[offset + 2] as u32;
    let b4 = data[offset + 3] as u32;
    return b1 | (b2 << 8) | (b3 << 16) | (b4 << 24);
}

pub fn get_u16_from_data(data: &Vec<u8>, offset: usize) -> u16 {
    let b1 = data[offset] as u16;
    let b2 = data[offset + 1] as u16;
    return b1 | (b2 << 8);
}

pub fn get_file_from_cdfh(data: &Vec<u8>, offset: usize) -> Result<(File, usize), ZipError> {
    let uncompressed_size = get_u32_from_data(data, offset + 24) as usize;
    let filename_length = get_u16_from_data(data, offset + 28) as usize;
    let extra_length = get_u16_from_data(data, offset + 30) as usize;
    let comment_length = get_u16_from_data(data, offset + 32) as usize;
    let file_offset = get_u32_from_data(data, offset + 42) as usize;
    let filename_bytes = &data[(offset + 46)..(offset + 46 + filename_length)];
    let filename = match std::str::from_utf8(filename_bytes) {
        Ok(s) => s.to_string(),
        Err(e) => return Err(ZipError::CorruptFile(format!("Not valid UTF ({})", e)))
    };
    let cdfh_size = 46 + filename_length + extra_length + comment_length;
    let lfh_filename_length = get_u16_from_data(data, file_offset + 26);
    let lfh_extra_length = get_u16_from_data(data, file_offset + 28);
    let lfh_size = (30 + lfh_filename_length + lfh_extra_length) as usize;
    
    let mut file_data = Vec::with_capacity(uncompressed_size);
    for i in 0..uncompressed_size {
        file_data.push(data[file_offset + lfh_size + i]);
    }

    let file = File::new(filename, Rc::new(file_data), None);
    return Ok((file, cdfh_size));
}

pub fn from_zip_archive(zip: &Vec<u8>) -> Result<Vec<File>, ZipError> {
    let mut files = Vec::new();

    let eocd_start = find_eocd(zip)?;
    let records_amount = if eocd_start + 8 < zip.len() {
        get_u16_from_data(zip, eocd_start + 6)
    } else {
        return Err(ZipError::CorruptFile("EOCD is missing total number of records".to_string()));
    };
    let central_directory_offset = if eocd_start + 16 < zip.len() {
        get_u32_from_data(zip, eocd_start + 12) as usize
    } else {
        return Err(ZipError::CorruptFile("EOCD is missing central directory offset".to_string()));
    };

    let mut offset = 0usize;
    for _ in 0..records_amount {
        let i = central_directory_offset + offset;
        let (file, cdfh_size) = get_file_from_cdfh(zip, i)?;
        files.push(file);
        offset += cdfh_size;
    }

    return Ok(files);
}