[![LICENSE](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE.txt)
[![](https://github.com/icsmw/bstorage/actions/workflows/push_and_pr_master.yml/badge.svg)](https://github.com/icsmw/bstorage/actions/workflows/push_and_pr_master.yml)
![Crates.io](https://img.shields.io/crates/v/bstorage)

`bstorage` is a lightweight library for storing data in binary form.

`bstorage` creates a file storage and allows writing/reading any data into it. The only requirement for the data is the implementation of `serde::Deserialize`, `serde::Serialize`.

```rust
use bstorage::Storage;
use serde::{Deserialize, Serialize};
use std::env::temp_dir;
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct MyRecord {
    field_a: String,
    field_b: Option<u8>,
}

// Set storage path
let storage_path = temp_dir().join(Uuid::new_v4().to_string());
// Create storage or open existed one
let mut storage = Storage::create(storage_path).expect("Storage created");
let my_record = MyRecord {
    field_a: "Hello World!".to_owned(),
    field_b: Some(255),
};
// Save record into storage
storage
    .set("my_record", &my_record)
    .expect("Record is saved");
// Read record from storage
let recovered: MyRecord = storage
    .get("my_record")
    .expect("Record is read")
    .expect("Record exists");
assert_eq!(my_record, recovered)
```

## When to use and when not to

`bstorage` is a good choice for:
- saving application settings
- saving temporary data
- other uses

`bstorage` is not the best choice if:
- you are looking for something like a lightweight database; `bstorage` is not a database
- searching through the array of saved data is required
- direct access to the data by the user is implied (e.g., toml, json, and other text formats)
- large amounts of data (1 GB or more) need to be saved

## How it works

`bstorage` uses the `bincode` crate for serializing and deserializing data. Each record is saved as a separate file within a directory specified when creating/opening the storage. Therefore, the number of records will be equivalent to the number of files in the storage.

The strategy of storing each record in a separate file is driven by the need to ensure maximum performance when writing data to the storage. If all data (all records) were stored in a single file, the seemingly simple task of "updating one record" would not be straightforward. On the file system level, data is stored in blocks, and "cleanly" replacing part of a file's content would require intervening in the file system's operations, which in most cases is an unnecessary complication. The simplest, most reliable, and effective method would be to overwrite the entire file. However, this approach leads to storage size issues. With large storage sizes, overwriting the entire storage becomes very costly.

This is why `bstorage` creates a separate file for each record, allowing for the fastest possible updating of each record without the need to overwrite the entire storage.

### How to transfer the storage

Transferring the storage can be done by copying the entire contents of the storage directory. However, in some situations, this can be quite inconvenient, especially if the data needs to be transferred over a network.

`bstorage` includes a `Bundle` trait that allows "packing" the entire storage into a single file, and then "unpacking" it back into its "normal" state.

```rust
use bstorage::{Bundle, Storage};
use serde::{Deserialize, Serialize};
use std::{
    env::temp_dir,
    fs::{remove_dir_all, remove_file},
};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct MyRecord {
    field_a: String,
    field_b: Option<u8>,
}

// Set storage path
let storage_path = temp_dir().join(Uuid::new_v4().to_string());
// Create storage or open existed one
let mut storage = Storage::create(&storage_path).expect("Storage created");
let my_record = MyRecord {
    field_a: "Hello World!".to_owned(),
    field_b: Some(255),
};
// Save record into storage
storage
    .set("my_record", &my_record)
    .expect("Record is saved");
// Pack storage into file
let packed = temp_dir().join(Uuid::new_v4().to_string());
storage.pack(&packed).expect("Storage packed");
// Remove origin storage
remove_dir_all(storage_path).expect("Origin storage has been removed");
drop(storage);
// Unpack storage
let storage =
    Storage::unpack(&packed).expect("Storage unpacked");
// Remove bundle file
remove_file(packed).expect("Bundle file removed");
// Read record from unpacked storage
let recovered: MyRecord = storage
    .get("my_record")
    .expect("Record is read")
    .expect("Record exists");
assert_eq!(my_record, recovered)
```