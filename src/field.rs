use crate::{fs, E};
use serde::{Deserialize, Serialize};
use std::{
    fs::remove_file,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

const STORAGE_FILE_EXT: &str = "bstorage";

#[derive(Debug)]
pub struct Field {
    path: PathBuf,
    cwd: PathBuf,
}

impl Field {
    pub fn restore<P: AsRef<Path>>(cwd: P, path: P) -> Self {
        Self {
            cwd: fs::as_path_buf(cwd),
            path: fs::as_path_buf(path),
        }
    }
    pub fn create<P: AsRef<Path>>(cwd: P) -> Self {
        let cwd = fs::as_path_buf(cwd);
        let path = cwd.join(format!("{}.{STORAGE_FILE_EXT}", Uuid::new_v4()));
        Self { cwd, path }
    }
    pub fn get<V: for<'a> Deserialize<'a> + 'static>(&self) -> Result<Option<V>, E> {
        let mut buffer = Vec::new();
        fs::read(&self.path)?.read_to_end(&mut buffer)?;
        Ok(Some(bincode::deserialize::<V>(&buffer)?))
    }
    pub fn set<V: Serialize + 'static>(&self, value: &V) -> Result<(), E> {
        let mut file = fs::create(&self.path)?;
        let buffer = bincode::serialize(&value)?;
        file.write_all(&buffer)?;
        Ok(())
    }
    pub fn extract(&self) -> Result<Vec<u8>, E> {
        let mut buffer: Vec<u8> = Vec::new();
        fs::read(&self.path)?.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
    pub fn remove(&self) -> Result<(), E> {
        if self.path.exists() {
            remove_file(&self.path)?;
        }
        Ok(())
    }
    pub fn file_name(&self) -> Result<String, E> {
        Ok(self
            .path
            .file_name()
            .ok_or(E::InvalidPath(self.path.clone()))?
            .to_string_lossy()
            .to_string())
    }
    pub fn size(&self) -> Result<u64, E> {
        Ok(self.path.metadata()?.len())
    }
}
