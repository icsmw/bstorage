use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("IO Error: {0}")]
    IO(#[from] io::Error),
    #[error("Serialize/Deserialize error: {0}")]
    Bincode(bincode::ErrorKind),
    #[error("Given path isn't a folder: {0}")]
    PathIsNotFolder(PathBuf),
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),
    #[error("Storage file {0} doesn't exist")]
    PackageFileDoesNotExist(PathBuf),
    #[error("Storage file {0} is invalid")]
    PackageFileInvalid(PathBuf),
    #[error("Fail to get parent of package file")]
    NoParentOfStorageFile,
    #[error("unknown data store error")]
    Unknown,
}

impl From<bincode::ErrorKind> for E {
    fn from(err: bincode::ErrorKind) -> Self {
        E::Bincode(err)
    }
}

impl From<Box<bincode::ErrorKind>> for E {
    fn from(err: Box<bincode::ErrorKind>) -> Self {
        E::Bincode(*err)
    }
}
