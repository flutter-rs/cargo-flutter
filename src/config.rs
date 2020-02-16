use crate::error::Error;
use serde::Deserialize;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub engine_version: Option<String>,
    pub apk: crate::package::apk::TomlApk,
    pub appimage: crate::package::appimage::TomlAppImage,
}

impl Config {
    pub fn parse_from_toml(path: &Path) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path)?;
        let config: TomlConfig = toml::from_str(&contents)?;
        let metadata = config.package.metadata.unwrap_or_default();
        Ok(Self {
            name: config.package.name,
            version: config.package.version,
            engine_version: metadata.flutter.unwrap_or_default().engine_version,
            apk: metadata.apk.unwrap_or_default(),
            appimage: metadata.appimage.unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct TomlConfig {
    package: TomlPackage,
}

#[derive(Debug, Clone, Deserialize)]
struct TomlPackage {
    name: String,
    version: String,
    metadata: Option<TomlMetadata>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct TomlMetadata {
    flutter: Option<TomlFlutter>,
    apk: Option<crate::package::apk::TomlApk>,
    appimage: Option<crate::package::appimage::TomlAppImage>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct TomlFlutter {
    engine_version: Option<String>,
}
