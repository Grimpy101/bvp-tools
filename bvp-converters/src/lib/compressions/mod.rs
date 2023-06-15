use crate::errors::CompressionError;

pub mod lz4s;

#[derive(Clone, Copy)]
pub enum CompressionType {
    None,
    LZ4S
}

impl CompressionType {
    pub fn to_string(&self) -> String {
        match self {
            CompressionType::LZ4S => return "lz4s".to_string(),
            CompressionType::None => return "raw".to_string()
        }
    }

    pub fn from_string(s: &str) -> Result<Self, CompressionError> {
        return match s {
            "LZ4S" | "lz4s" => Ok(Self::LZ4S),
            "RAW" | "raw" => Ok(Self::None),
            _ => Err(CompressionError::Unsupported(s.to_string()))
        }
    }

    pub fn compress(&self, source: Vec<u8>) -> Vec<u8> {
        return match self {
            CompressionType::LZ4S => lz4s::compress_lz4s(&source),
            CompressionType::None => source
        }
    }

    pub fn decompress(&self, source: &Vec<u8>, size: usize) -> Vec<u8> {
        return match self {
            CompressionType::LZ4S => lz4s::decompress_lz4s(&source, size),
            CompressionType::None => source.to_vec()
        }
    }
}