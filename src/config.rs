use crate::cargo::Cargo;
use crate::error::Error;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TomlConfig {
    pub package: TomlPackage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TomlPackage {
    pub name: String,
    pub metadata: Option<TomlMetadata>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct TomlMetadata {
    pub flutter: Option<TomlFlutter>,
    pub appimage: Option<crate::package::appimage::TomlAppImage>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct TomlFlutter {
    pub engine_version: Option<String>,
}

impl TomlConfig {
    pub fn load(cargo: &Cargo) -> Result<Self, Error> {
        let package = if let Some(package) = cargo.package() {
            cargo
                .workspace()
                .members()
                .find(|pkg| pkg.name().as_str() == package)
                .ok_or(Error::PackageNotMember)?
        } else {
            cargo.workspace().current()?
        };

        let bytes = std::fs::read(package.manifest_path())?;
        let string = std::str::from_utf8(&bytes)?;
        Ok(toml::from_str(string)?)
    }

    pub fn metadata(&self) -> TomlMetadata {
        self.package.metadata.clone().unwrap_or_default()
    }
}

impl TomlMetadata {
    pub fn engine_version(&self) -> Option<String> {
        self.flutter.clone().unwrap_or_default().engine_version
    }
}
