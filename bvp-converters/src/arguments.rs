use std::{fs};

use tinyjson::JsonValue;

use crate::{vector3::Vector3, formats::{Format}, json_aux};
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
    pub output_file: String,
    pub dimensions: Vector3<u32>,
    pub block_dimensions: Vector3<u32>,
    pub input_format: Format
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

    let input_file = json_aux::get_string_from_json(&json["inputFile"])?;
    let output_file = json_aux::get_string_from_json(&json["outputFile"])?;
    let dimensions = json_aux::get_u32_dimensions_from_json(&json["dimensions"])?;
    let block_dimensions = json_aux::get_u32_dimensions_from_json(&json["blockDimensions"])?;
    let input_format = Format::from_json(&json["format"])?;

    let arguments = Parameters {
        input_file,
        output_file,
        dimensions,
        block_dimensions,
        input_format
    };
    return Ok(arguments);
}