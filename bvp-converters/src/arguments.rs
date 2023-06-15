use std::{fs, collections::HashMap, path::Path};

use thiserror::Error;
use tinyjson::JsonValue;

use bvp::{vector3::Vector3, formats::{Format}, json_aux, archives::ArchiveEnum, compressions::CompressionType, errors::{JsonError, FormatError, ArchiveError, CompressionError}};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid config JSON: `{0}`")]
    InvalidJson(#[source] JsonError),
    #[error("Could not parse config JSON: `{0}`")]
    ParsingFailure(String),
    #[error("Cannot open config file: `{0}`")]
    CannotOpenFile(String),
    #[error("Error retrieving format from config: `{0}`")]
    FormatError(FormatError),
    #[error("Error retrieving archive type from config: `{0}`")]
    ArchiveError(ArchiveError),
    #[error("Error retrieving compression scheme from config: `{0}`")]
    CompressionError(CompressionError),
}

pub struct Parameters {
    pub input_file: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub semantic_type: Option<String>,
    pub volume_scale: Vector3<f32>,
    pub voxel_scale: Option<Vector3<f32>>,
    pub output_file: String,
    pub dimensions: Vector3<u32>,
    pub block_dimensions: Vector3<u32>,
    pub input_format: Format,
    pub archive: ArchiveEnum,
    pub compression: CompressionType,
    pub author: Option<String>,
    pub copyright: Option<String>,
    pub acquisition_time: Option<String>
}

pub fn parse_config(filepath: &str) -> Result<Parameters, ConfigError> {
    let contents = match fs::read_to_string(filepath) {
        Ok(c) => c,
        Err(e) => {
            return Err(ConfigError::CannotOpenFile(e.to_string()));
        },
    };

    let json: JsonValue = match contents.parse() {
        Ok(j) => j,
        Err(e) => {
            return Err(ConfigError::ParsingFailure(e.to_string()));
        },
    };
    let hashmap: HashMap<_, _> = match json.try_into() {
        Ok(h) => h,
        Err(e) => {
            return Err(ConfigError::ParsingFailure(e.to_string()));
        }
    };

    let input_file = json_aux::get_string_from_json(&hashmap["inputFile"]).map_err(|x| ConfigError::InvalidJson(x))?;
    let output_file = json_aux::get_string_from_json(&hashmap["outputFile"]).map_err(|x| ConfigError::InvalidJson(x))?;
    let dimensions = json_aux::get_u32_dimensions_from_json(&hashmap["dimensions"]).map_err(|x| ConfigError::InvalidJson(x))?;
    let block_dimensions = json_aux::get_u32_dimensions_from_json(&hashmap["blockDimensions"]).map_err(|x| ConfigError::InvalidJson(x))?;
    let input_format = Format::from_json(&hashmap["format"]).map_err(|x| ConfigError::FormatError(x))?;
    let archive = match hashmap.get("archive") {
        Some(s) => {
            match json_aux::get_string_from_json(s) {
                Ok(a) => ArchiveEnum::from_string(a).map_err(|x| ConfigError::ArchiveError(x))?,
                Err(e) => return Err(ConfigError::InvalidJson(e))
            }
        },
        None => ArchiveEnum::None
    };
    let compression = match hashmap.get("compression") {
        Some(s) => {
            let s = json_aux::get_string_from_json(s).map_err(|x| ConfigError::InvalidJson(x))?;
            CompressionType::from_string(&s).map_err(|x| ConfigError::CompressionError(x))?
        },
        None => CompressionType::None
    };
    let name = match hashmap.get("name") {
        Some(s) => {
            let s = json_aux::get_string_from_json(s).map_err(|x| ConfigError::InvalidJson(x))?;
            Some(s)
        },
        None => {
            let file2path = Path::new(&input_file);
            if file2path.file_stem().is_some() {
                Some(file2path.file_stem().unwrap().to_string_lossy().to_string())
            } else {
                None
            }
        }
    };
    let description = match hashmap.get("description") {
        Some(s) => {
            let s = json_aux::get_string_from_json(s).map_err(|x| ConfigError::InvalidJson(x))?;
            Some(s)
        },
        None => None
    };
    let semantic_type = match hashmap.get("semanticType") {
        Some(s) => {
            let s = json_aux::get_string_from_json(s).map_err(|x| ConfigError::InvalidJson(x))?;
            Some(s)
        },
        None => None
    };
    let volume_scale = match hashmap.get("volumeScale") {
        Some(s) => {
            Vector3::<f32>::from_json(s).map_err(|x| ConfigError::InvalidJson(x))?
        },
        None => Vector3::<f32>{ x: 1.0f32, y: 1.0f32, z: 1.0f32 }
    };
    let voxel_scale = match hashmap.get("voxelScale") {
        Some(s) => {
            Some(Vector3::<f32>::from_json(s).map_err(|x| ConfigError::InvalidJson(x))?)
        },
        None => None
    };
    let author = match hashmap.get("author") {
        Some(s) => {
            Some(json_aux::get_string_from_json(s).map_err(|x| ConfigError::InvalidJson(x))?)
        },
        None => None
    };
    let copyright = match hashmap.get("copyright") {
        Some(s) => {
            Some(json_aux::get_string_from_json(s).map_err(|x| ConfigError::InvalidJson(x))?)
        },
        None => None
    };
    let acquisition_time = match hashmap.get("acquisitionTime") {
        Some(s) => {
            Some(json_aux::get_string_from_json(s).map_err(|x| ConfigError::InvalidJson(x))?)
        },
        None => None
    };

    let arguments = Parameters {
        input_file,
        output_file,
        dimensions,
        block_dimensions,
        input_format,
        archive,
        compression,
        name,
        description,
        semantic_type,
        volume_scale,
        voxel_scale,
        author,
        copyright,
        acquisition_time
    };
    return Ok(arguments);
}