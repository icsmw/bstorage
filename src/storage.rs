use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{fs, Field, Map, E};

#[derive(Debug)]
pub struct BinStorage {
    pub(crate) map: Map,
    pub(crate) bundle: Option<PathBuf>,
    pub(crate) cwd: PathBuf,
    pub(crate) fields: HashMap<String, Field>,
}

impl BinStorage {
    pub fn open<P: AsRef<Path>>(cwd: P) -> Result<Self, E> {
        if !cwd.as_ref().exists() {
            return Err(E::PathIsNotFolder(fs::as_path_buf(cwd)));
        }
        let map = Map::new(&cwd);
        let fields = map.read()?;
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
        self.map.write(&self.fields)
    }

    pub fn clear(&mut self) -> Result<(), E> {
        for (_, field) in self.fields.iter() {
            field.remove()?;
        }
        self.fields.clear();
        self.map.write(&self.fields)
    }

    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
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
