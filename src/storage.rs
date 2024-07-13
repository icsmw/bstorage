use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::create_dir,
    io::{Read, Seek, SeekFrom, Write},
    mem,
    path::{Path, PathBuf},
};

use crate::{fs, Field, E};

const MAP_FILE_NAME: &str = "map.bstorage";
const UNPACKED_EXT: &str = "unpacked";
const U64_SIZE: usize = mem::size_of::<u64>();

#[derive(Debug)]
pub struct BinStorage {
    map: PathBuf,
    bundle: Option<PathBuf>,
    cwd: PathBuf,
    fields: HashMap<String, Field>,
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

    pub fn unpack<P: AsRef<Path>>(bundle: P) -> Result<Self, E> {
        let bundle = fs::as_path_buf(bundle);
        if !bundle.exists() || !bundle.is_file() {
            return Err(E::PackageFileDoesNotExist(bundle));
        }
        let mut cwd = bundle.clone();
        cwd.set_extension(UNPACKED_EXT);
        if !cwd.exists() {
            create_dir(&cwd)?;
        }
        let mut file = fs::read(&bundle)?;
        if bundle.metadata()?.len() < U64_SIZE as u64 {
            return Err(E::PackageFileInvalid(bundle));
        }
        let mut buffer = [0u8; U64_SIZE];
        file.read_exact(&mut buffer)?;
        let map_pos = u64::from_le_bytes(buffer) as usize;
        let mut buffer: Vec<u8> = Vec::new();
        file.seek(SeekFrom::Start(map_pos as u64))?;
        file.read_to_end(&mut buffer)?;
        let location: HashMap<String, (String, u64, u64)> = bincode::deserialize(&buffer)?;
        let mut map: HashMap<String, String> = HashMap::new();
        for (key, (filename, from, to)) in location {
            if to < from {
                warn!("Record \"{key}\" has invalid position. Record will be skipped");
                continue;
            }
            let size = (to - from) as usize;
            let mut buffer = vec![0; size];
            file.seek(SeekFrom::Start(from))?;
            file.read_exact(&mut buffer)?;
            let mut record = fs::create(cwd.join(&filename))?;
            record.write_all(&buffer)?;
            map.insert(key, filename);
        }
        let mut map_file = fs::create(cwd.join(MAP_FILE_NAME))?;
        let buffer = bincode::serialize(&map)?;
        map_file.write_all(&buffer)?;
        let mut storage = Self::open(cwd)?;
        storage.bundle = Some(bundle);
        Ok(storage)
    }

    pub fn pack<P: AsRef<Path>>(&mut self, bundle: P) -> Result<(), E> {
        let mut location: Vec<(String, String, u64, u64)> = Vec::new();
        let mut cursor = U64_SIZE as u64;
        let fields = self.fields.iter().collect::<Vec<(&String, &Field)>>();
        for (key, field) in fields.iter() {
            let size = field.size()?;
            if size == 0 {
                continue;
            }
            location.push((
                key.to_owned().clone(),
                field.file_name()?,
                cursor,
                cursor + size,
            ));
            cursor += size;
        }
        let map = bincode::serialize(&location)?;
        let mut bundle = fs::create(bundle)?;
        bundle.write_all(&cursor.to_le_bytes())?;
        for (_, field) in fields.iter() {
            bundle.write_all(&field.extract()?)?;
        }
        bundle.write_all(&map)?;
        Ok(())
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
