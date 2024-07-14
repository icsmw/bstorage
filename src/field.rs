use crate::{fs, E};
use serde::{Deserialize, Serialize};
use std::{
    fs::remove_file,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

const STORAGE_FILE_EXT: &str = "bstorage";
/// `Field` is a struct representing a single field stored in a binary file within the storage system.
#[derive(Debug)]
pub struct Field {
    path: PathBuf,
}

impl Field {
    /// Restores a `Field` from the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - A path reference to the file of the field.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns an instance of `Field`.
    pub fn restore<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: fs::as_path_buf(path),
        }
    }

    /// Creates a new `Field` in the specified directory.
    ///
    /// # Arguments
    ///
    /// * `cwd` - A path reference to the current working directory.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns a newly created instance of `Field`.
    pub fn create<P: AsRef<Path>>(cwd: P) -> Self {
        let cwd = fs::as_path_buf(cwd);
        let path = cwd.join(format!("{}.{STORAGE_FILE_EXT}", Uuid::new_v4()));
        Self { path }
    }

    /// Retrieves the value of the field. Returns None of case of deserializing error.
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// * `Result<Option<V>, E>` - Returns the deserialized value of the field or an error.
    pub fn get<V: for<'a> Deserialize<'a> + 'static>(&self) -> Result<Option<V>, E> {
        let mut buffer = Vec::new();
        fs::read(&self.path)?.read_to_end(&mut buffer)?;
        bincode::deserialize::<V>(&buffer)
            .map(|v| Some(v))
            .or_else(|_| Ok(None))
    }

    /// Retrieves the value of the field. Returns error in case of case of deserializing error.
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// * `Result<Option<V>, E>` - Returns the deserialized value of the field or an error.
    pub fn get_sensitive<V: for<'a> Deserialize<'a> + 'static>(&self) -> Result<Option<V>, E> {
        let mut buffer = Vec::new();
        fs::read(&self.path)?.read_to_end(&mut buffer)?;
        Ok(Some(bincode::deserialize::<V>(&buffer)?))
    }

    /// Sets the value of the field.
    ///
    /// # Arguments
    ///
    /// * `value` - A reference to the value to be stored.
    ///
    /// # Returns
    ///
    /// * `Result<(), E>` - Returns Ok(()) if successful, or an error.
    pub fn set<V: Serialize + 'static>(&self, value: &V) -> Result<(), E> {
        let mut file = fs::create(&self.path)?;
        let buffer = bincode::serialize(&value)?;
        file.write_all(&buffer).map_err(|e| e.into())
    }

    /// Extracts the binary content of the field.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<u8>, E>` - Returns the binary content as a vector of bytes, or an error.
    pub fn extract(&self) -> Result<Vec<u8>, E> {
        let mut buffer: Vec<u8> = Vec::new();
        fs::read(&self.path)?.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    /// Removes the field from the storage.
    ///
    /// # Returns
    ///
    /// * `Result<(), E>` - Returns Ok(()) if successful, or an error.
    pub fn remove(&self) -> Result<(), E> {
        if self.path.exists() {
            remove_file(&self.path)?;
        }
        Ok(())
    }

    /// Retrieves the file name of the field.
    ///
    /// # Returns
    ///
    /// * `Result<String, E>` - Returns the file name as a string, or an error.
    pub fn file_name(&self) -> Result<String, E> {
        Ok(self
            .path
            .file_name()
            .ok_or(E::InvalidPath(self.path.clone()))?
            .to_string_lossy()
            .to_string())
    }

    /// Retrieves the size of the field in bytes.
    ///
    /// # Returns
    ///
    /// * `Result<u64, E>` - Returns the size of the field in bytes, or an error.
    pub fn size(&self) -> Result<u64, E> {
        Ok(self.path.metadata()?.len())
    }
}
