use tinyjson::JsonValue;

use crate::{vector3::Vector3, errors::JsonError};

pub fn get_u32_from_json(j: &JsonValue) -> Result<u32, JsonError> {
    match j {
        JsonValue::Number(n) => return Ok(*n as u32),
        _ => return Err(JsonError::NotANumber(j.clone()))
    };
}

pub fn get_f32_from_json(j: &JsonValue) -> Result<f32, JsonError> {
    match j {
        JsonValue::Number(n) => return Ok(*n as f32),
        _ => return Err(JsonError::NotANumber(j.clone()))
    };
}

pub fn get_u32_dimensions_from_json(j: &JsonValue) -> Result<Vector3<u32>, JsonError> {
    match j {
        JsonValue::Array(a) => {
            let x = get_u32_from_json(&a[0])?;
            let y = get_u32_from_json(&a[1])?;
            let z = get_u32_from_json(&a[2])?;
            return Ok(Vector3::from_xyz(x, y, z));
        },
        _ => return Err(JsonError::NotAnArray(j.clone()))
    }
}

pub fn get_string_from_json(j: &JsonValue) -> Result<String, JsonError> {
    match j {
        JsonValue::String(s) => return Ok(s.clone()),
        _ => return Err(JsonError::NotAString(j.clone()))
    }
}

pub fn get_array_from_json(j: &JsonValue) -> Result<Vec<JsonValue>, JsonError> {
    match j {
        JsonValue::Array(a) => return Ok(a.to_vec()),
        _ => return Err(JsonError::NotAnArray(j.clone()))
    }
}

pub fn get_string_vec_from_json(j: &JsonValue) -> Result<Vec<String>, JsonError> {
    let mut vec = Vec::new();
    match j {
        JsonValue::Array(a) => {
            for el in a {
                match el {
                    JsonValue::String(s) => vec.push(s.clone()),
                    _ => return Err(JsonError::NotAString(el.clone()))
                }
            }
        },
        _ => return Err(JsonError::NotAnArray(j.clone()))
    }
    return Ok(vec);
}