use thiserror::Error;
use tinyjson::JsonValue;

use crate::vector3::Vector3;


#[derive(Error, Debug)]
pub enum PlacementError {
    #[error("Invalid JSON at placement (block `{0}`): `{1}`")]
    InvalidJson(usize, #[source] JsonError)
}

#[derive(Error, Debug)]
pub enum ModalityError {
    #[error("Invalid JSON at modality `{0}`: `{1}`")]
    InvalidJson(usize, #[source] JsonError)
}

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Invalid JSON at asset: `{0}`")]
    InvalidJson(#[source] JsonError)
}

#[derive(Error, Debug)]
pub enum BvpFileError {
    #[error("Error in asset: `{0}`")]
    AssetError(#[source] AssetError),
    #[error("Invalid JSON in BVP manifest: `{0}`")]
    InvalidJson(#[source] JsonError),
    #[error("Invalid manifest: `{0}`")]
    BrokenManifest(String),
    #[error("Block error: `{0}`")]
    BlockError(BlockError),
    #[error("Modality error: `{0}`")]
    ModalityError(ModalityError),
    #[error("Format error: `{0}`")]
    FormatError(FormatError)
}


#[derive(Error, Debug)]
pub enum BlockError {
    #[error("Block `{0}` does not have data")]
    NoData(usize),
    #[error("Formats of block `{0}` and block `{1}` do not match")]
    FormatMismatch(usize, usize),
    #[error("Block `{0}`: start (`{1}`) is greater than end (`{2}`)")]
    StartGreaterThanEnd(usize, Vector3<u32>, Vector3<u32>),
    #[error("Block `{0}`: start (`{1}`) is out of bounds")]
    StartOutOfBounds(usize, Vector3<u32>),
    #[error("Block `{0}`: end (`{1}`) is out of bounds")]
    EndOutOfBounds(usize, Vector3<u32>),
    #[error("Block `{0}` is not on microblock boundary (`{1}` not divisible by `{2}`)")]
    BlockInvalidPosition(usize, Vector3<u32>, Vector3<u32>),
    #[error("Block `{0}` cannot contain whole microblocks (`{1}` not divisible by `{2}`)")]
    BlockInvalidSize(usize, Vector3<u32>, Vector3<u32>),
    #[error("Invalid compression scheme in block `{0}`: `{1}`")]
    InvalidCompression(usize, CompressionError),
    #[error("Invalid JSON at block `{0}`: `{1}`")]
    InvalidJson(usize, #[source] JsonError),
    #[error("Invalid placement at block `{0}`: `{1}`")]
    InvalidPlacement(usize, #[source] PlacementError)
}

#[derive(Error, Debug)]
pub enum JsonError {
    #[error("JSON value `{0:?}` is not a number")]
    NotANumber(JsonValue),
    #[error("JSON value `{0:?}` is not an array")]
    NotAnArray(JsonValue),
    #[error("JSON value `{0:?}` is not a string")]
    NotAString(JsonValue),
    #[error("JSON value `{0:?}` is not a 3D vector")]
    NotAVector3(JsonValue),
    #[error("JSON value `{0:?}` is not an object")]
    NotAnObject(JsonValue),
}

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("SAF error: `{0}`")]
    SafError(SafError),
    #[error("The provided archive format is not supported (`{0}`)")]
    NotImplemented(String),
    #[error("Archive file or folder does not exist (`{0}`)")]
    DoesNotExist(String),
    #[error("Cannot read file: `{0}`")]
    CannotRead(String),
    #[error("Not a valid file: `{0}`")]
    NotValidFile(String),
    #[error("Cannot write file: `{0}`")]
    CannotWrite(String)
}

#[derive(Error, Debug)]
pub enum SafError {
    #[error("SAF identifier is not valid")]
    NotValidIdentifier,
    #[error("Not a valid SAF file")]
    BrokenFile,
    #[error("SAF manifest is corrupt: `{0}`")]
    ManifestCorrupt(String),
    #[error("Invalid JSON: `{0}`")]
    InvalidJson(JsonError)
}

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Unsupported compression (`{0}`)")]
    Unsupported(String)
}

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Invalid mono format component type (`{0}`)")]
    MonoInvalidComponentType(String),
    #[error("Invalid JSON for format: `{0}`")]
    InvalidJson(JsonError),
    #[error("Unsupported format family: `{0}`")]
    UnsupportedFormatFamily(String)
}