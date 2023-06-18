use std::{fs, path::Path};
use crate::{file::File, errors::ArchiveError};

pub mod saf;
pub mod zip;
pub mod unarchived;

pub enum ArchiveEnum {
    SAF,
    ZIP,
    None
}

impl ArchiveEnum {
    /// Takes the type of archive to use and writes files.
    /// * `files` - a vector of files to write
    /// * `output_filepath` - a filepath to write to; should be a file in case of archive or a folder in case of unarchived output
    pub fn write_files(&self, files: &Vec<File>, output_filepath: String) -> Result<(), ArchiveError> {
        match self {
            Self::SAF => {
                let saf = match saf::to_saf_archive(files) {
                    Ok(s) => s,
                    Err(e) => return Err(ArchiveError::SafError(e))
                };
                match fs::write(output_filepath, saf) {
                    Ok(_) => (),
                    Err(e) => return Err(ArchiveError::CannotWrite(e.to_string()))
                };
            },
            Self::ZIP => {
                let zip = match zip::to_zip_archive(files) {
                    Ok(z) => z,
                    Err(e) => return Err(ArchiveError::ZipError(e))
                };
                match fs::write(output_filepath, zip) {
                    Ok(_) => (),
                    Err(e) => return Err(ArchiveError::CannotWrite(e.to_string()))
                }
            },
            Self::None => {
                for file in files {
                    match file.write() {
                        Ok(_) => (),
                        Err(e) => return Err(ArchiveError::CannotWrite(e.to_string()))
                    };
                }
            }
        };
        return Ok(());
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
