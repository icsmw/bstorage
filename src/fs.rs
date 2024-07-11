use std::{
    fs::{File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

pub fn create<P: AsRef<Path>>(filename: P) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)
}

pub fn read<P: AsRef<Path>>(filename: P) -> io::Result<File> {
    OpenOptions::new().read(true).open(filename)
}

pub fn create_or_open<P: AsRef<Path>>(filename: P) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .truncate(false)
        .open(filename)
}

pub fn as_path_buf<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().to_path_buf()
}
