use log::{debug, warn};
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::{fs, Field, E};

pub(crate) const MAP_FILE_NAME: &str = "map.bstorage";

#[derive(Debug)]
pub struct Map {
    cwd: PathBuf,
    path: PathBuf,
}

impl Map {
    pub fn new<P: AsRef<Path>>(cwd: P) -> Self {
        Self {
            cwd: fs::as_path_buf(&cwd),
            path: fs::as_path_buf(&cwd).join(MAP_FILE_NAME),
        }
    }
    pub fn read(&self) -> Result<HashMap<String, Field>, E> {
        if !self.path.exists() {
            debug!("Storage's map file will be created: {:?}", self.path);
        }
        let mut file = fs::create_or_open(&self.path)?;
        let mut fields: HashMap<String, Field> = HashMap::new();
        if file.metadata()?.len() > 0 {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            let decoded: HashMap<String, String> = bincode::deserialize(&buffer)?;
            for (key, filename) in decoded.into_iter() {
                let file_path = self.cwd.join(&filename);
                if !file_path.exists() {
                    warn!("File \"{filename}\" for key \"{key}\" doesn't exist");
                    continue;
                }
                fields.insert(key, Field::restore(&file_path));
            }
        }
        Ok(fields)
    }

    pub fn write(&mut self, fields: &HashMap<String, Field>) -> Result<(), E> {
        let mut files: HashMap<String, String> = HashMap::new();
        for (key, field) in fields.iter() {
            let file_name = field.file_name()?;
            files.insert(key.to_owned(), file_name);
        }
        let buffer = bincode::serialize(&files)?;
        let mut map = fs::create(&self.path)?;
        map.write_all(&buffer)?;
        Ok(())
    }
}
