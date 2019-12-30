use std::path::PathBuf;

pub mod appimage;

pub struct Package {
    name: String,
    bin: Vec<PathBuf>,
    lib: Vec<PathBuf>,
    asset: Vec<PathBuf>,
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

    pub fn bins(&self) -> &[PathBuf] {
        &self.bin
    }

    pub fn libs(&self) -> &[PathBuf] {
        &self.lib
    }

    pub fn assets(&self) -> &[PathBuf] {
        &self.asset
    }

    pub fn add_bin(&mut self, path: PathBuf) {
        self.bin.push(path);
    }

    pub fn add_lib(&mut self, path: PathBuf) {
        self.lib.push(path);
    }

    pub fn add_asset(&mut self, path: PathBuf) {
        self.asset.push(path);
    }
}
