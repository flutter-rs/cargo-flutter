use crate::engine::Build;
use crate::project::Project;
use std::path::{Path, PathBuf};

pub mod apk;
pub mod appimage;

pub struct Package {
    root_dir: PathBuf,
    out_dir: PathBuf,
    name: String,
    version: String,
    triple: String,
    build: Build,
    bin: Vec<Item>,
    lib: Vec<Item>,
    asset: Vec<Item>,
}

impl Package {
    pub fn from_project(project: &Project) -> Self {
        Self {
            name: project.config.name.clone(),
            version: project.config.version.clone(),
            triple: project.target_triple.clone(),
            build: project.build,
            root_dir: project.root_dir.clone(),
            out_dir: project.out_dir.clone(),
            bin: Default::default(),
            lib: Default::default(),
            asset: Default::default(),
        }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn out_dir(&self) -> &Path {
        &self.out_dir
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn triple(&self) -> &str {
        &self.triple
    }

    pub fn build(&self) -> Build {
        self.build
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

impl From<&PathBuf> for Item {
    fn from(path: &PathBuf) -> Self {
        path.clone().into()
    }
}

impl From<&Path> for Item {
    fn from(path: &Path) -> Self {
        path.to_path_buf().into()
    }
}
