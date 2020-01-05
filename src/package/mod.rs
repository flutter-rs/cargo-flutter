use std::path::{Path, PathBuf};

pub mod apk;
pub mod appimage;

pub struct Package {
    name: String,
    bin: Vec<Item>,
    lib: Vec<Item>,
    asset: Vec<Item>,
}

pub struct Item {
    path: PathBuf,
    name: String,
}

impl Item {
    pub fn new(path: PathBuf, name: String) -> Self {
        Self { path, name }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl From<PathBuf> for Item {
    fn from(path: PathBuf) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        Self { path, name }
    }
}

impl Package {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            bin: Default::default(),
            lib: Default::default(),
            asset: Default::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bins(&self) -> &[Item] {
        &self.bin
    }

    pub fn libs(&self) -> &[Item] {
        &self.lib
    }

    pub fn assets(&self) -> &[Item] {
        &self.asset
    }

    pub fn add_bin<T: Into<Item>>(&mut self, item: T) {
        self.bin.push(item.into());
    }

    pub fn add_lib<T: Into<Item>>(&mut self, item: T) {
        self.lib.push(item.into());
    }

    pub fn add_asset<T: Into<Item>>(&mut self, item: T) {
        self.asset.push(item.into());
    }
}
