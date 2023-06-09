use std::{fs, path::Path};
use crate::{file::File, errors::ArchiveError};

use self::{saf::SAFWriter, zip::ZIPWriter, unarchived::RawFilesWriter};

pub mod saf;
pub mod zip;
pub mod unarchived;

pub trait ArchiveWriter {
    fn append_file(&mut self, file: &File) -> Result<(), String>;
    fn finish(&self, path: String) -> Result<(), String>;
}

pub enum ArchiveEnum {
    SAF,
    ZIP,
    None
}

impl ArchiveEnum {
    pub fn return_writer(&self) -> Box<dyn ArchiveWriter + Send> {
        match self {
            Self::SAF => {
                return Box::new(SAFWriter::new());
            },
            Self::ZIP => {
                return Box::new(ZIPWriter::new());
            },
            Self::None => {
                return Box::new(RawFilesWriter::new());
            }
        }
    }

    pub fn from_string(str: String) -> Result<Self, ArchiveError> {
        // Be aware that the check first converts the string to lowercase!
        return match str.to_lowercase().as_str() {
            "saf" => Ok(ArchiveEnum::SAF),
            "zip" => Ok(ArchiveEnum::ZIP),
            "none" => Ok(ArchiveEnum::None),
            _ => return Err(ArchiveError::NotImplemented(str))
        }
    }

    /// Reads the archive file/folder and returns raw files inside.
    /// * `filepath` - path to file/folder to read
    pub fn read_archive(&self, filepath: &Path) -> Result<Vec<File>, ArchiveError> {
        if filepath.is_dir() {
            return unarchived::from_folder(filepath);
        } else if filepath.is_file() {
            return match self {
                ArchiveEnum::None => unarchived::from_manifest_file(&filepath),
                ArchiveEnum::SAF => {
                    let contents = match fs::read(filepath) {
                        Ok(v) => v,
                        Err(e) => return Err(ArchiveError::CannotRead(e.to_string()))
                    };
                    saf::from_saf_archive(&contents).map_err(|x| ArchiveError::SafError(x))
                },
                ArchiveEnum::ZIP => {
                    let contents = match fs::read(filepath) {
                        Ok(v) => v,
                        Err(e) => return Err(ArchiveError::CannotRead(e.to_string()))
                    };
                    zip::from_zip_archive(&contents).map_err(|x| ArchiveError::ZipError(x))
                }
            }
        }
        return Err(ArchiveError::NotValidFile(filepath.to_string_lossy().to_string()));
    }
}
