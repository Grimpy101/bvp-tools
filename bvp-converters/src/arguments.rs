use std::{fs, collections::HashMap, path::Path};

use tinyjson::JsonValue;

use crate::{vector3::Vector3, formats::{Format}, json_aux, archives::ArchiveEnum, compression::CompressionType};
/*
pub enum ArgType {
    String(String),
    Uint(u32),
    Int(i32),
    Float(f32),
    Dimensions3Uint([u32; 3]),
    Json(HashMap<String, JsonValue>)
}

impl ToString for ArgType {
    fn to_string(&self) -> String {
        return match self {
            ArgType::String(s) => s.clone(),
            ArgType::Uint(u) => u.to_string(),
            ArgType::Int(i) => i.to_string(),
            ArgType::Float(f) => f.to_string(),
            ArgType::Dimensions3Uint(d) => format!("{}x{}x{}",d[0], d[1], d[2]),
            ArgType::Json(j) => format!("{:?}", j)
        }
    }
}

pub enum ArgLabelType {
    String,
    Uint,
    Int,
    Float,
    Dimensions3Uint,
    Json
}

impl ArgLabelType {
    fn parse_str(arguments: &Vec<String>, i: usize) -> Result<ArgType, String> {
        if arguments.len() <= i + 1 {
            return Err(format!("No value supplied for argument {}", arguments[i]));
        }

        return Ok(ArgType::String(arguments[i+1].clone()));
    }

    fn parse_i32(arguments: &Vec<String>, i: usize) -> Result<ArgType, String> {
        if arguments.len() <= i + 1 {
            return Err(format!("No value supplied for argument {}", arguments[i]));
        }

        return match arguments[i+1].parse::<i32>() {
            Ok(a) => {
                Ok(ArgType::Int(a))
            },
            Err(_) => {
                Err(format!("Invalid int value provided for argument {}", arguments[i]))
            },
        };
    }

    fn parse_u32(arguments: &Vec<String>, i: usize) -> Result<ArgType, String> {
        if arguments.len() <= i + 1 {
            return Err(format!("No value supplied for argument {}", arguments[i]));
        }

        return match arguments[i+1].parse::<u32>() {
            Ok(a) => {
                Ok(ArgType::Uint(a))
            },
            Err(_) => {
                Err(format!("Invalid uint value provided for argument {}", arguments[i]))
            },
        };
    }

    fn parse_f32(arguments: &Vec<String>, i: usize) -> Result<ArgType, String> {
        if arguments.len() <= i + 1 {
            return Err(format!("No value supplied for argument {}", arguments[i]));
        }

        return match arguments[i+1].parse::<f32>() {
            Ok(a) => {
                Ok(ArgType::Float(a))
            },
            Err(_) => {
                Err(format!("Invalid float value provided for argument {}", arguments[i]))
            },
        };
    }

    fn parse_dims_u32(arguments: &Vec<String>, i: usize) -> Result<ArgType, String> {
        if arguments.len() <= i + 3 {
            return Err(format!("Not enough values for argument {}", arguments[i]));
        }

        let x = match arguments[i+1].parse::<u32>() {
            Ok(x) => x,
            Err(_) => {
                return Err(format!("Could not parse first element for argument {}", arguments[i]));
            },
        };

        let y = match arguments[i+2].parse::<u32>() {
            Ok(y) => y,
            Err(_) => {
                return Err(format!("Could not parse second element for argument {}", arguments[i]));
            },
        };

        let z = match arguments[i+3].parse::<u32>() {
            Ok(z) => z,
            Err(_) => {
                return Err(format!("Could not parse third element for argument {}", arguments[i]));
            },
        };

        return Ok(ArgType::Dimensions3Uint([x, y, z]));
    }

    fn parse_json(arguments: &Vec<String>, i: usize) -> Result<ArgType, String> {
        if arguments.len() <= i + 1 {
            return Err(format!("Not enough values for argument {}", arguments[i]));
        }

        let parsed: JsonValue = match arguments[i+1].parse() {
            Ok(p) => p,
            Err(e) => {
                return Err(format!("Could not parse json for argument {} ({})", arguments[i], e));
            }
        };

        let object: HashMap<_, _> = match parsed.try_into() {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("Failed to retrieve value of argument {} ({})", arguments[i], e));
            },
        };

        return Ok(ArgType::Json(object))
    }

    pub fn parse(&self, arguments: &Vec<String>, i: usize) -> Result<ArgType, String> {
        match self {
            ArgLabelType::String => {
                return ArgLabelType::parse_str(arguments, i);
            },
            ArgLabelType::Uint => {
                return ArgLabelType::parse_u32(arguments, i);
            },
            ArgLabelType::Int => {
                return ArgLabelType::parse_i32(arguments, i);
            },
            ArgLabelType::Float => {
                return ArgLabelType::parse_f32(arguments, i);
            },
            ArgLabelType::Dimensions3Uint => {
                return ArgLabelType::parse_dims_u32(arguments, i);
            },
            ArgLabelType::Json => {
                return ArgLabelType::parse_json(arguments, i);
            }
        };
    }
}

pub struct ArgLabel {
    name: String,
    tp: ArgLabelType,
    description: String,
    required: bool
}

impl ArgLabel {
    pub fn new(name: &str, tp: ArgLabelType, description: &str, required: bool) -> Self {
        return Self {
            name: name.to_string(),
            tp,
            description: description.to_string(),
            required
        }
    }
}

pub fn parse_args(labels: Vec<ArgLabel>, program_name: &str, program_description: &str) -> Result<HashMap<String, ArgType>, String> {

    let mut map: HashMap<String, ArgType> = HashMap::new();
    let mut required: Vec<&str> = Vec::new();

    let arguments: Vec<String> = env::args().collect();

    for label in &labels {
        if label.required {
            required.push(&label.name);
        }
    }

    for i in 0..arguments.len() {
        let argument = &arguments[i];

        if argument == "--help" {
            print!("{}\n\n{}\n\n", program_name, program_description);
            for label in &labels {
                print!("--{}:\t{}\n", label.name, label.description);
            }
            exit(0);  // This is not ideal but will do for now...
        }

        for li in 0..labels.len() {
            let label = &labels[li];
            if &format!("--{}", label.name) == argument {
                match label.tp.parse(&arguments, i) {
                    Ok(t) => {
                        map.insert(label.name.clone(), t)
                    },
                    Err(e) => {
                        return Err(e);
                    }
                };
                if label.required {
                    required.remove(li);
                }
                break;
            }
        }
    }

    if required.len() > 0 {
        return Err(format!("Missing required arguments: {:?}", required));
    }

    return Ok(map);
}
*/
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

pub fn parse_config(filepath: &str) -> Result<Parameters, String> {
    let contents = match fs::read_to_string(filepath) {
        Ok(c) => c,
        Err(_) => {
            return Err("Could not read config file".to_string());
        },
    };

    let json: JsonValue = match contents.parse() {
        Ok(j) => j,
        Err(e) => {
            return Err(format!("Could not read JSON: {}", e));
        },
    };
    let hashmap: HashMap<_, _> = match json.try_into() {
        Ok(h) => h,
        Err(_) => {
            return Err("Not valid JSON".to_string());
        }
    };

    let input_file = json_aux::get_string_from_json(&hashmap["inputFile"])?;
    let output_file = json_aux::get_string_from_json(&hashmap["outputFile"])?;
    let dimensions = json_aux::get_u32_dimensions_from_json(&hashmap["dimensions"])?;
    let block_dimensions = json_aux::get_u32_dimensions_from_json(&hashmap["blockDimensions"])?;
    let input_format = Format::from_json(&hashmap["format"])?;
    let archive = match hashmap.get("archive") {
        Some(s) => {
            let s = json_aux::get_string_from_json(s)?;
            match s.as_str() {
                "SAF" => ArchiveEnum::SAF,
                "None" => ArchiveEnum::None,
                _ => {
                    return Err("Archive format not supported".to_string());
                }
            }
        },
        None => ArchiveEnum::None
    };
    let compression = match hashmap.get("compression") {
        Some(s) => {
            let s = json_aux::get_string_from_json(s)?;
            match s.as_str() {
                "LZ4S" => CompressionType::LZ4S,
                "None" => CompressionType::None,
                _ => {
                    return Err("Compression type not supported".to_string());
                }
            }
        },
        None => CompressionType::None
    };
    let name = match hashmap.get("name") {
        Some(s) => {
            let s = json_aux::get_string_from_json(s)?;
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
            let s = json_aux::get_string_from_json(s)?;
            Some(s)
        },
        None => None
    };
    let semantic_type = match hashmap.get("semanticType") {
        Some(s) => {
            let s = json_aux::get_string_from_json(s)?;
            Some(s)
        },
        None => None
    };
    let volume_scale = match hashmap.get("volumeScale") {
        Some(s) => {
            Vector3::<f32>::from_json(s)?
        },
        None => Vector3::<f32>{ x: 1.0f32, y: 1.0f32, z: 1.0f32 }
    };
    let voxel_scale = match hashmap.get("voxelScale") {
        Some(s) => {
            Some(Vector3::<f32>::from_json(s)?)
        },
        None => None
    };
    let author = match hashmap.get("author") {
        Some(s) => {
            Some(json_aux::get_string_from_json(s)?)
        },
        None => None
    };
    let copyright = match hashmap.get("copyright") {
        Some(s) => {
            Some(json_aux::get_string_from_json(s)?)
        },
        None => None
    };
    let acquisition_time = match hashmap.get("acquisitionTime") {
        Some(s) => {
            Some(json_aux::get_string_from_json(s)?)
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