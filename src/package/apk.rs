use crate::engine::Build;
use crate::error::Error;
use crate::package::Package;
use android_build_tools::apk::ApkConfig;
use android_build_tools::cargo::VersionCode;
use android_build_tools::config::{Config, Metadata};
use android_build_tools::ndk::Ndk;
use android_build_tools::target::Target;

pub type TomlApk = Metadata;

pub struct Apk {
    apk_id: u8,
    ext: Option<String>,
    metadata: Metadata,
}

impl Apk {
    pub fn new(metadata: Metadata, apk_id: u8, ext: Option<&str>) -> Self {
        Self {
            apk_id,
            ext: ext.map(|s| s.to_string()),
            metadata,
        }
    }

    pub fn build(&self, package: &Package) -> Result<(), Error> {
        let ndk = Ndk::from_env()?;
        let assets = package
            .assets()
            .get(0)
            .map(|p| p.path().to_str().unwrap().to_string());
        let target = Target::from_rust_triple(package.triple())?;
        let package_label = if let Some(ext) = &self.ext {
            format!("{}-{}", package.name(), ext)
        } else {
            package.name().to_string()
        };
        let key = ndk.debug_key()?;
        let config = Config {
            ndk,
            build_dir: package.out_dir().join("apk"),
            package_name: format!("rust.flutter.{}", package.name().replace("-", "_")),
            package_label,
            version_name: package.version().to_string(),
            version_code: VersionCode::from_semver(package.version())?.to_code(self.apk_id),
            split: self.ext.clone(),
            debuggable: package.build() == Build::Debug,
            target_name: package.name().replace("-", "_"),
            assets,
            res: None,
        };
        let config = ApkConfig::from_config(config, self.metadata.clone());
        let apk = config.create_apk()?;
        for lib in package.libs() {
            apk.add_lib(lib.path(), target)?;
        }
        apk.align()?.sign(key)?;
        Ok(())
    }
}
