use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::{fs, Field, E};

const MAP_FILE_NAME: &str = "map.bstorage";

#[derive(Debug)]
pub struct BinStorage {
    pub(crate) map: PathBuf,
    pub(crate) bundle: Option<PathBuf>,
    pub(crate) cwd: PathBuf,
    pub(crate) fields: HashMap<String, Field>,
}

impl BinStorage {
    pub fn open<P: AsRef<Path>>(cwd: P) -> Result<Self, E> {
        if !cwd.as_ref().exists() {
            return Err(E::PathIsNotFolder(fs::as_path_buf(cwd)));
        }
        let map = fs::as_path_buf(&cwd).join(MAP_FILE_NAME);
        if !map.exists() {
            debug!("Storage's map file will be created: {map:?}");
        }
        let mut file = fs::create_or_open(&map)?;
        let mut fields: HashMap<String, Field> = HashMap::new();
        if file.metadata()?.len() > 0 {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            let decoded: HashMap<String, String> = bincode::deserialize(&buffer)?;
            for (key, filename) in decoded.into_iter() {
                let file_path = cwd.as_ref().join(&filename);
                if !file_path.exists() {
                    warn!("File \"{filename}\" for key \"{key}\" doesn't exist");
                    continue;
                }
                fields.insert(key, Field::restore(&file_path));
            }
        }
        Ok(Self {
            map,
            bundle: None,
            fields,
            cwd: fs::as_path_buf(cwd),
        })
    }

    pub fn get<V: for<'a> Deserialize<'a> + 'static, K: AsRef<str>>(
        &self,
        key: K,
    ) -> Result<Option<V>, E> {
        let Some(field) = self.fields.get(key.as_ref()) else {
            return Ok(None);
        };
        field.get::<V>()
    }

    pub fn get_or_default<V: for<'a> Deserialize<'a> + 'static + Default, K: AsRef<str>>(
        &self,
        key: K,
    ) -> Result<V, E> {
        Ok(self.get(key)?.unwrap_or(V::default()))
    }

    pub fn has<K: AsRef<str>>(&self, key: K) -> bool {
        self.fields.contains_key(key.as_ref())
    }

    pub fn set<V: Serialize + 'static, K: AsRef<str>>(
        &mut self,
        key: K,
        value: &V,
    ) -> Result<(), E> {
        let field = if let Some(field) = self.fields.remove(key.as_ref()) {
            field
        } else {
            Field::create(&self.cwd)
        };
        field.set::<V>(value)?;
        self.fields.insert(key.as_ref().to_owned(), field);
        self.write_map()
    }

    pub fn clear(&mut self) -> Result<(), E> {
        for (_, field) in self.fields.iter() {
            field.remove()?;
        }
        self.fields.clear();
        self.write_map()
    }

    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    fn write_map(&mut self) -> Result<(), E> {
        let mut files: HashMap<String, String> = HashMap::new();
        for (key, field) in self.fields.iter() {
            let file_name = field.file_name()?;
            files.insert(key.to_owned(), file_name);
        }
        let buffer = bincode::serialize(&files)?;
        let mut map = fs::create(&self.map)?;
        map.write_all(&buffer)?;
        Ok(())
    }
}

pub struct BinStorageIter<'a> {
    keys: Vec<&'a String>,
    pos: usize,
}

impl<'a> Iterator for BinStorageIter<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.keys.len() {
            None
        } else {
            self.pos += 1;
            Some(self.keys[self.pos - 1])
        }
    }
}

impl<'a> IntoIterator for &'a BinStorage {
    type Item = &'a String;
    type IntoIter = BinStorageIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BinStorageIter {
            keys: self.fields.keys().collect(),
            pos: 0,
        }
    }
}
