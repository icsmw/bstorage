use std::{
    fs::{File, OpenOptions},
    io,
    path::{Path, PathBuf},
};
/// Creates a new file or truncates an existing file and opens it for writing.
///
/// # Arguments
///
/// * `filename` - A path reference to the file to be created or truncated.
///
/// # Returns
///
/// * `io::Result<File>` - Returns a `File` handle if successful, or an error.
pub fn create<P: AsRef<Path>>(filename: P) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)
}

/// Opens an existing file for reading.
///
/// # Arguments
///
/// * `filename` - A path reference to the file to be opened.
///
/// # Returns
///
/// * `io::Result<File>` - Returns a `File` handle if successful, or an error.
pub fn read<P: AsRef<Path>>(filename: P) -> io::Result<File> {
    OpenOptions::new().read(true).open(filename)
}

/// Opens a file for reading and writing, creating it if it doesn't exist.
///
/// # Arguments
///
/// * `filename` - A path reference to the file to be created or opened.
///
/// # Returns
///
/// * `io::Result<File>` - Returns a `File` handle if successful, or an error.
pub fn create_or_open<P: AsRef<Path>>(filename: P) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .truncate(false)
        .open(filename)
}

/// Converts a path reference to a `PathBuf`.
///
/// # Arguments
///
/// * `path` - A path reference to be converted.
///
/// # Returns
///
/// * `PathBuf` - Returns the path as a `PathBuf`.
pub fn as_path_buf<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().to_path_buf()
}
