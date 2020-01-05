use crate::cargo::Cargo;
use crate::package::Package;
use cargo::core::manifest::TargetKind;
use cargo_apk::{AndroidBuildTarget, AndroidConfig, BuildTarget, SharedLibraries, SharedLibrary};
use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct TomlApk {}

pub struct Apk {
    toml: AndroidConfig,
}

impl Apk {
    pub fn new(toml: AndroidConfig) -> Self {
        Self { toml }
    }

    pub fn build(
        &self,
        cargo: &Cargo,
        package: &Package,
        _sign: bool,
        abi: AndroidBuildTarget,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = self.toml.clone();
        config.default_target_config.assets =
            Some(package.assets()[0].path().to_str().unwrap().to_string());
        let mut libs = SharedLibraries {
            shared_libraries: Default::default(),
        };
        let target = BuildTarget::new(package.name().to_string(), TargetKind::Bin);
        for lib in package.libs() {
            libs.shared_libraries.insert(
                target.clone(),
                SharedLibrary {
                    abi,
                    path: lib.path().to_owned(),
                    filename: lib.name().to_owned(),
                },
            );
        }

        cargo_apk::build_apks(&config, &cargo.build_dir(), libs)?;
        Ok(())
    }
}
