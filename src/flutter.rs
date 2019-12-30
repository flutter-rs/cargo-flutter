use crate::cargo::Cargo;
use crate::engine::Build;
use crate::error::Error;
use cargo::core::Workspace;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

pub struct Flutter {
    root: PathBuf,
}

impl Flutter {
    pub fn new() -> Result<Self, Error> {
        let root = if let Ok(root) = std::env::var("FLUTTER_ROOT") {
            PathBuf::from(root)
        } else {
            let flutter = which::which("flutter").expect("flutter not found");
            let flutter = std::fs::canonicalize(flutter)?;
            flutter
                .parent()
                .ok_or(Error::FlutterNotFound)?
                .parent()
                .ok_or(Error::FlutterNotFound)?
                .to_owned()
        };
        log::info!("FLUTTER_ROOT {}", root.display());
        Ok(Flutter { root })
    }

    pub fn engine_version(&self) -> Result<String, Error> {
        let version = if let Ok(v) = std::env::var("FLUTTER_ENGINE_VERSION") {
            v
        } else {
            let path = self
                .root
                .join("bin")
                .join("internal")
                .join("engine.version");
            std::fs::read_to_string(path).map(|v| v.trim().to_owned())?
        };
        log::info!("FLUTTER_ENGINE_VERSION {}", version);
        Ok(version)
    }

    pub fn bundle(&self, cargo: &Cargo, build: Build) -> ExitStatus {
        let flag = match build {
            Build::Debug => "--debug",
            Build::Release => "--release",
            Build::Profile => "--profile",
        };

        Command::new("flutter")
            .current_dir(cargo.workspace().root())
            .arg("build")
            .arg("bundle")
            .arg(flag)
            .arg("--track-widget-creation")
            .arg("--asset-dir")
            .arg(cargo.build_dir().join("flutter_assets"))
            .arg("--depfile")
            .arg(cargo.build_dir().join("snapshot_blob.bin.d"))
            .status()
            .expect("flutter build bundle")
    }

    pub fn attach(&self, workspace: &Workspace, debug_uri: &str) -> ExitStatus {
        let debug_uri = format!("--debug-uri={}", debug_uri);
        Command::new("flutter")
            .current_dir(workspace.root())
            .arg("attach")
            .arg("--device-id=flutter-tester")
            .arg(debug_uri)
            .status()
            .expect("Success")
    }
}
