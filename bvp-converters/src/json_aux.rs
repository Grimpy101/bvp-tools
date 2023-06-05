use tinyjson::JsonValue;

use crate::{vector3::Vector3};

pub fn get_u32_from_json(j: &JsonValue) -> Result<u32, String> {
    match j {
        JsonValue::Number(n) => {
            return Ok(*n as u32);
        },
        _ => {
            return Err("JSON parsing error: Not a number".to_string());
        }
    };
}

pub fn get_u32_dimensions_from_json(j: &JsonValue) -> Result<Vector3<u32>, String> {
    match j {
        JsonValue::Array(a) => {
            let x = get_u32_from_json(&a[0])?;
            let y = get_u32_from_json(&a[1])?;
            let z = get_u32_from_json(&a[2])?;
            return Ok(Vector3::from_xyz(x, y, z));
        },
        _ => {
            return Err("JSON parsing error: Not an array".to_string());
        }
    }
}

pub fn get_string_from_json(j: &JsonValue) -> Result<String, String> {
    match j {
        JsonValue::String(s) => {
            return Ok(s.clone());
        },
        _ => {
            return Err("JSON parsing error: Not a string".to_string());
        }
    }
}

pub fn get_array_from_json(j: &JsonValue) -> Result<Vec<JsonValue>, String> {
    match j {
        JsonValue::Array(a) => {
            return Ok(a.to_vec());
        },
        _ => {
            return Err("JSON parsing error: Not an array".to_string());
        }
    }
}