use log::warn;
use std::{
    collections::HashMap,
    fs::create_dir,
    io::{Read, Seek, SeekFrom, Write},
    mem,
    path::Path,
};

use crate::{fs, map, Field, Storage, E};

/// Default extention of bundle file
const UNPACKED_EXT: &str = "unpacked";
const U64_SIZE: usize = mem::size_of::<u64>();

/// Transferring the storage can be done by copying the entire contents of the storage directory. However,
/// in some situations, this can be quite inconvenient, especially if the data needs to be transferred over
/// a network.
///
/// The `Bundle` trait helps to resolve this issue and provides methods for packing and unpacking the storage
/// into/from a single file.
///
/// # Example
/// ```rust
/// use bstorage::{Bundle, Storage};
/// use serde::{Deserialize, Serialize};
/// use std::{
///     env::temp_dir,
///     fs::{remove_dir_all, remove_file},
/// };
/// use uuid::Uuid;
///
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// pub struct MyRecord {
///     field_a: String,
///     field_b: Option<u8>,
/// }
///
/// // Set storage path
/// let storage_path = temp_dir().join(Uuid::new_v4().to_string());
/// // Create storage or open existed one
/// let mut storage = Storage::create(&storage_path).expect("Storage created");
/// let my_record = MyRecord {
///     field_a: "Hello World!".to_owned(),
///     field_b: Some(255),
/// };
/// // Save record into storage
/// storage
///    .set("my_record", &my_record)
///    .expect("Record is saved");
/// // Pack storage into file
/// let packed = temp_dir().join(Uuid::new_v4().to_string());
/// storage.pack(&packed).expect("Storage packed");
/// // Remove origin storage
/// remove_dir_all(storage_path).expect("Origin storage has been removed");
/// drop(storage);
/// // Unpack storage
/// let storage =
///     Storage::unpack(&packed).expect("Storage unpacked");
/// // Remove bundle file
/// remove_file(packed).expect("Bundle file removed");
/// // Read record from unpacked storage
/// let recovered: MyRecord = storage
///    .get("my_record")
///    .expect("Record is read")
///    .expect("Record exists");
/// assert_eq!(my_record, recovered)
/// ```
pub trait Bundle {
    /// Unpacks the storage from the specified bundle file.
    ///
    /// # Arguments
    ///
    /// * `bundle` - A path reference to the bundle file.
    ///
    /// # Returns
    ///
    /// * `Result<Storage, E>` - Returns the unpacked `Storage` instance or an error.
    fn unpack<P: AsRef<Path>>(bundle: P) -> Result<Storage, E>;

    /// Packs the storage into the specified bundle file.
    ///
    /// # Arguments
    ///
    /// * `bundle` - A path reference to the bundle file.
    ///
    /// # Returns
    ///
    /// * `Result<(), E>` - Returns Ok(()) if successful, or an error.
    fn pack<P: AsRef<Path>>(&mut self, bundle: P) -> Result<(), E>;
}

impl Bundle for Storage {
    /// Unpacks the storage from the specified bundle file.
    ///
    /// This method reads the bundle file, extracts individual records, and writes them
    /// to the storage directory specified by changing the extension of the bundle file.
    ///
    /// # Arguments
    ///
    /// * `bundle` - A path reference to the bundle file.
    ///
    /// # Returns
    ///
    /// * `Result<Self, E>` - Returns the unpacked `Storage` instance or an error.
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
        let mut map_file = fs::create(cwd.join(map::MAP_FILE_NAME))?;
        let buffer = bincode::serialize(&map)?;
        map_file.write_all(&buffer)?;
        Self::open(cwd)
    }

    /// Packs the storage into the specified bundle file.
    ///
    /// This method serializes all records into a single file for easy transfer and storage.
    ///
    /// # Arguments
    ///
    /// * `bundle` - A path reference to the bundle file.
    ///
    /// # Returns
    ///
    /// * `Result<(), E>` - Returns Ok(()) if successful, or an error.
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
