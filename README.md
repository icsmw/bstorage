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
- if you are looking for something like a lightweight database; bstorage is not a database
- if you need fast search within the array of saved data without copying data (again, bstorage is not a database)
- direct access to the data by the user is implied (e.g., toml, json, and other text formats)
- large amounts of data (1 GB or more) need to be saved

## How it works

`bstorage` uses the `bincode` crate for serializing and deserializing data. Each record is saved as a separate file within a directory specified when creating/opening the storage. Therefore, the number of records will be equivalent to the number of files in the storage.

The strategy of storing each record in a separate file is driven by the need to ensure maximum performance when writing data to the storage. If all data (all records) were stored in a single file, the seemingly simple task of "updating one record" would not be straightforward. On the file system level, data is stored in blocks, and "cleanly" replacing part of a file's content would require intervening in the file system's operations, which in most cases is an unnecessary complication. The simplest, most reliable, and effective method would be to overwrite the entire file. However, this approach leads to storage size issues. With large storage sizes, overwriting the entire storage becomes very costly.

This is why `bstorage` creates a separate file for each record, allowing for the fastest possible updating of each record without the need to overwrite the entire storage.

## How to transfer the storage

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

## Searching Records in Storage

To implement searching for records in the storage, you should use the Search trait, which provides access to two methods: find and filter.

```rust
use bstorage::{Search, Storage, E};
use serde::{Deserialize, Serialize};
use std::{env::temp_dir, fs::remove_dir_all};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct A {
    a: u8,
    b: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct B {
    c: u32,
    d: Option<bool>,
}

let mut storage = Storage::create(temp_dir().join(Uuid::new_v4().to_string())).unwrap();
let a = [
    A {
        a: 0,
        b: String::from("one"),
    },
    A {
        a: 1,
        b: String::from("two"),
    },
    A {
        a: 2,
        b: String::from("three"),
    },
];
let b = [
    B {
        c: 0,
        d: Some(true),
    },
    B {
        c: 1,
        d: Some(false),
    },
    B {
        c: 2,
        d: Some(true),
    },
];
let mut i = 0;
for a in a.iter() {
    storage.set(i.to_string(), a).unwrap();
    i += 1;
}
for b in b.iter() {
    storage.set(i.to_string(), b).unwrap();
    i += 1;
}
let (_key, found) = storage.find(|v: &A| &a[0] == v).unwrap().expect("Record found");
assert_eq!(found, a[found.a as usize]);
let (_key, found) = storage.find(|v: &B| &b[0] == v).unwrap().expect("Record found");
assert_eq!(found, b[found.c as usize]);
assert!(storage.find(|v: &A| v.a > 254).unwrap().is_none());
storage.clear().unwrap();
remove_dir_all(storage.cwd()).unwrap();
```

And example of filtering

```rust
use bstorage::{Search, Storage, E};
use serde::{Deserialize, Serialize};
use std::{env::temp_dir, fs::remove_dir_all};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct A {
    a: u8,
    b: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct B {
    c: u32,
    d: Option<bool>,
}

let mut storage = Storage::create(temp_dir().join(Uuid::new_v4().to_string())).unwrap();
let a = [
    A {
        a: 0,
        b: String::from("one"),
    },
    A {
        a: 1,
        b: String::from("two"),
    },
    A {
        a: 2,
        b: String::from("three"),
    },
];
let b = [
    B {
        c: 0,
        d: Some(true),
    },
    B {
        c: 1,
        d: Some(false),
    },
    B {
        c: 2,
        d: Some(true),
    },
];
let mut i = 0;
for a in a.iter() {
    storage.set(i.to_string(), a).unwrap();
    i += 1;
}
for b in b.iter() {
    storage.set(i.to_string(), b).unwrap();
    i += 1;
}
let found = storage.filter(|v: &A| v.a < 2).unwrap();
assert_eq!(found.len(), 2);
for (_key, found) in found.into_iter() {
    assert_eq!(found, a[found.a as usize]);
}
let found = storage.filter(|v: &B| v.c < 2).unwrap();
assert_eq!(found.len(), 2);
for (_key, found) in found.into_iter() {
    assert_eq!(found, b[found.c as usize]);
}
assert_eq!(storage.filter(|v: &A| v.a > 254).unwrap().len(), 0);
storage.clear().unwrap();
remove_dir_all(storage.cwd()).unwrap();
```

## Contributing

Contributions are welcome! Please read the short [Contributing Guide](CONTRIBUTING.md).