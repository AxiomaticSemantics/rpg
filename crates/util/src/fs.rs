use std::{
    fs::{self, File},
    io,
    path::Path,
};

pub fn open_read(path: &Path) -> Result<File, io::Error> {
    fs::OpenOptions::new().read(true).open(path)
}

pub fn open_write(path: &Path) -> Result<File, io::Error> {
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
}
