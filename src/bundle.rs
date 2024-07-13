use log::warn;
use std::{
    collections::HashMap,
    fs::create_dir,
    io::{Read, Seek, SeekFrom, Write},
    mem,
    path::Path,
};

use crate::{fs, BinStorage, Field, E};

const MAP_FILE_NAME: &str = "map.bstorage";
const UNPACKED_EXT: &str = "unpacked";
const U64_SIZE: usize = mem::size_of::<u64>();

pub trait Bundle {
    fn unpack<P: AsRef<Path>>(bundle: P) -> Result<BinStorage, E>;
    fn pack<P: AsRef<Path>>(&mut self, bundle: P) -> Result<(), E>;
}

impl Bundle for BinStorage {
    fn unpack<P: AsRef<Path>>(bundle: P) -> Result<Self, E> {
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

    fn pack<P: AsRef<Path>>(&mut self, bundle: P) -> Result<(), E> {
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
}
