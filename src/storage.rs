use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{create_dir, remove_file, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    mem,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::E;

pub(crate) const MAP_FILE_NAME: &str = "map.bstorage";
pub(crate) const STORAGE_FILE_EXT: &str = "bstorage";
pub(crate) const UNPACKED_EXT: &str = "unpacked";

const U64_SIZE: usize = mem::size_of::<u64>();

fn new_file<P: AsRef<Path>>(filename: P) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)
}
pub struct BinStorage {
    map_path: PathBuf,
    bundle: Option<PathBuf>,
    cwd: PathBuf,
    files: HashMap<String, PathBuf>,
}

impl BinStorage {
    pub fn open<P: AsRef<Path>>(cwd: P) -> Result<Self, E> {
        if !cwd.as_ref().exists() {
            return Err(E::PathIsNotFolder(cwd.as_ref().to_path_buf()));
        }
        let map_path = cwd.as_ref().to_path_buf().join(MAP_FILE_NAME);
        if !map_path.exists() {
            debug!("Storage's map file will be created: {map_path:?}");
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .truncate(false)
            .open(&map_path)?;
        let mut files: HashMap<String, PathBuf> = HashMap::new();
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
                files.insert(key, file_path);
            }
        }
        Ok(Self {
            map_path,
            bundle: None,
            files,
            cwd: cwd.as_ref().to_path_buf(),
        })
    }

    pub fn unpack<P: AsRef<Path>>(bundle: P) -> Result<Self, E> {
        let bundle = bundle.as_ref().to_path_buf();
        if !bundle.exists() || !bundle.is_file() {
            return Err(E::PackageFileDoesNotExist(bundle));
        }
        let mut cwd = bundle.clone();
        cwd.set_extension(UNPACKED_EXT);
        if !cwd.exists() {
            create_dir(&cwd)?;
        }
        let mut file = OpenOptions::new().read(true).open(&bundle)?;
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
            let mut record = new_file(cwd.join(&filename))?;
            record.write_all(&buffer)?;
            record.flush()?;
            map.insert(key, filename);
        }
        let mut map_file = new_file(cwd.join(MAP_FILE_NAME))?;
        let buffer = bincode::serialize(&map)?;
        map_file.write_all(&buffer)?;
        map_file.flush()?;
        let mut storage = Self::open(cwd)?;
        storage.bundle = Some(bundle);
        Ok(storage)
    }

    pub fn pack<P: AsRef<Path>>(&mut self, bundle_file: P) -> Result<(), E> {
        let mut location: Vec<(String, String, u64, u64)> = Vec::new();
        let mut cursor = U64_SIZE as u64;
        let files = self.files.iter().collect::<Vec<(&String, &PathBuf)>>();
        for (key, filepath) in files.iter() {
            let size = filepath.metadata()?.len();
            if size == 0 {
                continue;
            }
            location.push((
                key.to_owned().clone(),
                filepath
                    .file_name()
                    .ok_or(E::InvalidPath(filepath.to_owned().clone()))?
                    .to_string_lossy()
                    .to_string(),
                cursor,
                cursor + size,
            ));
            cursor += size;
        }
        let map = bincode::serialize(&location)?;
        let mut bundle_file = new_file(bundle_file)?;
        bundle_file.write_all(&cursor.to_le_bytes())?;
        for (_, filepath) in files.iter() {
            let mut file = OpenOptions::new().read(true).open(filepath)?;
            let mut buffer: Vec<u8> = Vec::new();
            file.read_to_end(&mut buffer)?;
            bundle_file.write_all(&buffer)?;
        }
        bundle_file.write_all(&map)?;
        bundle_file.flush()?;
        Ok(())
    }

    fn write_map(&mut self) -> Result<(), E> {
        let mut files: HashMap<String, String> = HashMap::new();
        for (key, filepath) in self.files.iter() {
            let Some(filename) = filepath.file_name() else {
                warn!("Fail get filename for entry \"{key}\"");
                continue;
            };
            files.insert(key.to_owned(), filename.to_string_lossy().to_string());
        }
        let buffer = bincode::serialize(&files)?;
        let mut map_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.map_path)?;
        map_file.write_all(&buffer)?;
        map_file.flush()?;
        Ok(())
    }

    pub fn get<V: for<'a> Deserialize<'a>, K: AsRef<str>>(&self, key: K) -> Result<Option<V>, E> {
        let Some(filename) = self.files.get(key.as_ref()) else {
            return Ok(None);
        };
        let mut file = OpenOptions::new().read(true).open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(Some(bincode::deserialize::<V>(&buffer)?))
    }

    pub fn get_or_default<V: for<'a> Deserialize<'a> + Default, K: AsRef<str>>(
        &self,
        key: K,
    ) -> Result<V, E> {
        Ok(self.get(key)?.unwrap_or(V::default()))
    }

    pub fn has<K: AsRef<str>>(&self, key: K) -> bool {
        self.files.contains_key(key.as_ref())
    }
    pub fn set<V: Serialize, K: AsRef<str>>(&mut self, key: K, value: &V) -> Result<(), E> {
        let filename = self
            .cwd
            .join(format!("{}.{STORAGE_FILE_EXT}", Uuid::new_v4()));
        let filename = self.files.get(key.as_ref()).unwrap_or(&filename).to_owned();
        self.files
            .insert(key.as_ref().to_owned(), filename.to_owned());
        let mut file = new_file(filename)?;
        let buffer = bincode::serialize(&value)?;
        file.write_all(&buffer)?;
        file.flush()?;
        self.write_map()
    }

    pub fn clear(&mut self) -> Result<(), E> {
        for (_, filepath) in self.files.iter() {
            remove_file(filepath)?;
        }
        self.files.clear();
        self.write_map()
    }

    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }
}
