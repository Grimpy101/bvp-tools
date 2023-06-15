use std::{collections::HashMap};

use tinyjson::JsonValue;

use crate::{vector3::Vector3, json_aux::{get_string_from_json, get_u32_from_json}, errors::{FormatError, JsonError}};

pub enum PrimitiveType {
    Int,
    Uint,
    Float
}

impl PrimitiveType {
    pub fn from_string(s: &str) -> Result<PrimitiveType, FormatError> {
        return match s {
            "u" => Ok(PrimitiveType::Uint),
            "i" => Ok(PrimitiveType::Int),
            "f" => Ok(PrimitiveType::Float),
            _ => Err(FormatError::MonoInvalidComponentType(s.to_string()))
        }
    }

    pub fn to_string(&self) -> String {
        return match self {
            PrimitiveType::Float => "f".to_string(),
            PrimitiveType::Int => "i".to_string(),
            PrimitiveType::Uint => "u".to_string()
        };
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

    pub fn from_hashmap(o: &HashMap<String, JsonValue>) -> Result<(FormatFamily, Vector3<u32>, u32), FormatError> {
        let count = match get_u32_from_json(&o["count"]) {
            Ok(c) => c,
            Err(e) => return Err(FormatError::InvalidJson(e))
        };
        let size = match get_u32_from_json(&o["size"]) {
            Ok(s) => s,
            Err(e) => return Err(FormatError::InvalidJson(e))
        };
        let tp = match get_string_from_json(&o["type"]) {
            Ok(t) => t,
            Err(e) => return Err(FormatError::InvalidJson(e))
        };
        let prim = PrimitiveType::from_string(&tp)?;
        let mono = MonoFormat::new(count, size, prim);
        let mono_family = FormatFamily::Mono(mono);
        let microblock_dimensions = Vector3::from_xyz(1, 1, 1);
        let microblock_size = count * size;
        return Ok((mono_family, microblock_dimensions, microblock_size));
    }
}

pub enum FormatFamily {
    Mono(MonoFormat)
}

impl FormatFamily {
    pub fn to_json(&self, hm: &mut HashMap<String, JsonValue>) {
        match self {
            FormatFamily::Mono(m) => {
                hm.insert("family".to_string(), "mono".to_string().into());
                hm.insert("count".to_string(), (m.count as f64).into());
                hm.insert("type".to_string(), m.tp.to_string().into());
                hm.insert("size".to_string(), (m.size as f64).into());
            }
        }
    }

    pub fn from_hashmap(o: &HashMap<String, JsonValue>) -> Result<(Self, Vector3<u32>, u32), FormatError> {
        let family = match get_string_from_json(&o["family"]) {
            Ok(f) => f,
            Err(e) => return Err(FormatError::InvalidJson(e))
        };

        return match family.as_str() {
            "mono" => {
                MonoFormat::from_hashmap(o)
            },
            _ => return Err(FormatError::UnsupportedFormatFamily(family))
        }
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

    pub fn to_json(&self) -> JsonValue {
        let mut hm = HashMap::new();
        hm.insert("microblockSize".to_string(), (self.microblock_size as f64).into());
        hm.insert("microblockDimensions".to_string(), self.microblock_dimensions.to_json());
        self.family.to_json(&mut hm);
        return hm.into();
    }

    /// Returns the size of an amount of microblocks in bytes.
    /// * `microblock_amount` - amount of microblocks
    pub fn count_microblocks(&self, microblock_amount: u32) -> u32 {
        return self.microblock_size * microblock_amount;
    }

    /// Returns the size of a block in bytes.
    /// * `dimensions` - dimensions of the block
    pub fn count_space(&self, dimensions: Vector3<u32>) -> u32 {
        let microblock_amount_vec = (dimensions / self.microblock_dimensions).to_u32();
        let microblock_amount = microblock_amount_vec.multiply_elements();
        return self.count_microblocks(microblock_amount);
    }

    pub fn from_json(j: &JsonValue) -> Result<Self, FormatError> {
        return match j {
            JsonValue::Object(o) => {
                let (family, mb_dim, mb_size) = FormatFamily::from_hashmap(o)?;
                let format = Self::new(mb_dim, mb_size, family);
                Ok(format)
            },
            _ => {
                Err(FormatError::InvalidJson(JsonError::NotAnObject(j.clone())))
            }
        }
    }
}