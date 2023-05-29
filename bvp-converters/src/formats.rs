use std::{collections::HashMap};

use tinyjson::JsonValue;

use crate::vector3::Vector3;

pub enum PrimitiveType {
    Int,
    Uint,
    Float
}

impl PrimitiveType {
    pub fn from_str(s: String) -> Result<PrimitiveType, String> {
        match s.as_str() {
            "u" => {
                return Ok(PrimitiveType::Uint);
            },
            "i" => {
                return Ok(PrimitiveType::Int);
            },
            "f" => {
                return Ok(PrimitiveType::Float);
            },
            _ => {
                return Err("Not a valid mono component type".to_string());
            }
        }
    }
}

pub struct MonoFormat {
    count: u32,
    tp: PrimitiveType,
    size: u32
}

impl MonoFormat {
    pub fn new(count: u32, size: u32, tp: PrimitiveType) -> Self {
        return Self { count, size, tp };
    }
}

pub enum FormatFamily {
    Mono(MonoFormat)
}

impl FormatFamily {
    pub fn to_string(&self) -> String {
        return match self {
            FormatFamily::Mono(_) => "mono".to_string(),
        };
    }
}

pub struct Format {
    pub microblock_dimensions: Vector3<u32>,
    pub microblock_size: u32,
    family: FormatFamily
}

impl Format {
    pub fn new(microblock_dimensions: Vector3<u32>, microblock_size: u32, family: FormatFamily) -> Self {
        return Self { microblock_dimensions, microblock_size, family };
    }

    pub fn to_json(&self) -> HashMap<String, JsonValue> {
        let mut hm = HashMap::new();
        hm.insert("family".to_string(), self.family.to_string().into());
        hm.insert("microblockSize".to_string(), (self.microblock_size as f64).into());
        hm.insert("microblockDimensions".to_string(), self.microblock_dimensions.to_f64_vec().into());
        return hm;
    }

    pub fn count_microblocks(&self, microblock_amount: u32) -> u32 {
        return self.microblock_size * microblock_amount;
    }

    pub fn count_space(&self, dimensions: Vector3<u32>) -> u32 {
        let microblock_amount_vec = (dimensions / self.microblock_dimensions).to_u32();
        let microblock_amount = microblock_amount_vec.multiply_elements();
        return self.count_microblocks(microblock_amount);
    }
}