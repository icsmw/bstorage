use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::create_dir,
    path::{Path, PathBuf},
};

use crate::{fs, Field, Map, E};

/// `Storage` is a struct for managing binary data storage. It utilizes the `bincode` crate for
/// serialization and deserialization of data. Each record is stored as a separate file within a specified directory.
/// The strategy of storing each record in a separate file is driven by the need to ensure maximum performance when
/// writing data to the storage. If all data (all records) were stored in a single file, the seemingly simple task
/// of "updating one record" would not be straightforward. On the file system level, data is stored in blocks, and
/// "cleanly" replacing part of a file's content would require intervening in the file system's operations, which
/// in most cases is an unnecessary complication. The simplest, most reliable, and effective method would be to
/// overwrite the entire file. However, this approach leads to storage size issues. With large storage sizes,
/// overwriting the entire storage becomes very costly.
///
/// This is why `bstorage` creates a separate file for each record, allowing for the fastest possible updating of
/// each record without the need to overwrite the entire storage.
#[derive(Debug)]
pub struct Storage {
    pub(crate) map: Map,
    pub(crate) cwd: PathBuf,
    pub(crate) fields: HashMap<String, Field>,
}

impl Storage {
    /// Creates a new storage if it does not exist and opens the storage.
    ///
    /// # Arguments
    ///
    /// * `cwd` - A path reference to the storage directory.
    ///
    /// # Returns
    ///
    /// * `Result<Self, E>` - Returns the created `Storage` instance or an error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bstorage::Storage;
    /// use serde::{Deserialize, Serialize};
    /// use std::env::temp_dir;
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
    /// let mut storage = Storage::create(storage_path).expect("Storage created");
    /// let my_record = MyRecord {
    ///     field_a: "Hello World!".to_owned(),
    ///     field_b: Some(255),
    /// };
    /// // Save record into storage
    /// storage
    ///     .set("my_record", &my_record)
    ///     .expect("Record is saved");
    /// // Read record from storage
    /// let recovered: MyRecord = storage
    ///     .get("my_record")
    ///     .expect("Record is read")
    ///     .expect("Record exists");
    /// assert_eq!(my_record, recovered)
    /// ```
    pub fn create<P: AsRef<Path>>(cwd: P) -> Result<Self, E> {
        if !cwd.as_ref().exists() {
            create_dir(&cwd)?;
        }
        Storage::open(cwd)
    }

    /// Opens an existing storage.
    ///
    /// # Arguments
    ///
    /// * `cwd` - A path reference to the storage directory.
    ///
    /// # Returns
    ///
    /// * `Result<Self, E>` - Returns the opened `Storage` instance or an error.
    pub fn open<P: AsRef<Path>>(cwd: P) -> Result<Self, E> {
        if !cwd.as_ref().exists() {
            return Err(E::PathIsNotFolder(fs::as_path_buf(cwd)));
        }
        let map = Map::new(&cwd);
        let fields = map.read()?;
        Ok(Self {
            map,
            fields,
            cwd: fs::as_path_buf(cwd),
        })
    }

    /// Retrieves a value associated with the specified key. Returns None of case of deserializing error.
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to the key as a string slice.
    ///
    /// # Returns
    ///
    /// * `Result<Option<V>, E>` - Returns the value if found, or None if not found, or an error.
    pub fn get<V: for<'a> Deserialize<'a> + 'static, K: AsRef<str>>(
        &self,
        key: K,
    ) -> Result<Option<V>, E> {
        let Some(field) = self.fields.get(key.as_ref()) else {
            return Ok(None);
        };
        field.get::<V>()
    }

    /// Retrieves a value associated with the specified key.Returns error in case of case of deserializing error.
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to the key as a string slice.
    ///
    /// # Returns
    ///
    /// * `Result<Option<V>, E>` - Returns the value if found, or None if not found, or an error.
    pub fn get_sensitive<V: for<'a> Deserialize<'a> + 'static, K: AsRef<str>>(
        &self,
        key: K,
    ) -> Result<Option<V>, E> {
        let Some(field) = self.fields.get(key.as_ref()) else {
            return Ok(None);
        };
        field.get_sensitive::<V>()
    }

    /// Retrieves a value associated with the specified key, or returns a default value if the key does not exist.
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to the key as a string slice.
    ///
    /// # Returns
    ///
    /// * `Result<V, E>` - Returns the value or the default value, or an error.
    pub fn get_or_default<V: for<'a> Deserialize<'a> + 'static + Default, K: AsRef<str>>(
        &self,
        key: K,
    ) -> Result<V, E> {
        Ok(self.get(key)?.unwrap_or(V::default()))
    }

    /// Checks if the specified key exists in the storage.
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to the key as a string slice.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns true if the key exists, false otherwise.
    pub fn has<K: AsRef<str>>(&self, key: K) -> bool {
        self.fields.contains_key(key.as_ref())
    }

    /// Sets a value for the specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to the key as a string slice.
    /// * `value` - A reference to the value to be stored.
    ///
    /// # Returns
    ///
    /// * `Result<(), E>` - Returns Ok(()) if successful, or an error.
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

    /// Removes the value associated with the specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to the key as a string slice.
    ///
    /// # Returns
    ///
    /// * `Result<bool, E>` - Returns true if the key was found and removed, false otherwise, or an error.
    pub fn remove<K: AsRef<str>>(&mut self, key: K) -> Result<bool, E> {
        let Some(field) = self.fields.get(key.as_ref()) else {
            return Ok(false);
        };
        field.remove()?;
        self.fields.remove(key.as_ref());
        self.map.write(&self.fields)?;
        Ok(true)
    }

    /// Clears all entries from the storage and removes bound files. This method will not remove a storage folder.
    ///
    /// # Returns
    ///
    /// * `Result<(), E>` - Returns Ok(()) if successful, or an error.
    pub fn clear(&mut self) -> Result<(), E> {
        for (_, field) in self.fields.iter() {
            field.remove()?;
        }
        self.fields.clear();
        self.map.write(&self.fields)
    }

    /// Returns the current working directory of the storage.
    ///
    /// # Returns
    ///
    /// * `&PathBuf` - A reference to the current working directory path buffer.
    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }
}

/// Iterator for iterating over keys in the storage.
pub struct StorageIter<'a> {
    keys: Vec<&'a String>,
    pos: usize,
}

impl<'a> Iterator for StorageIter<'a> {
    type Item = &'a String;

    /// Advances the iterator and returns the next key.
    ///
    /// # Returns
    ///
    /// * `Option<&'a String>` - The next key, or None if iteration is finished.
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.keys.len() {
            None
        } else {
            self.pos += 1;
            Some(self.keys[self.pos - 1])
        }
    }
}

impl<'a> IntoIterator for &'a Storage {
    type Item = &'a String;
    type IntoIter = StorageIter<'a>;

    /// Creates an iterator over the keys in the storage.
    ///
    /// # Returns
    ///
    /// * `StorageIter<'a>` - An iterator over the keys in the storage.
    fn into_iter(self) -> Self::IntoIter {
        StorageIter {
            keys: self.fields.keys().collect(),
            pos: 0,
        }
    }
}
