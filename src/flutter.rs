use crate::engine::{Build, Engine};
use crate::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Flutter {
    root: PathBuf,
}

impl Flutter {
    pub fn new() -> Result<Self, Error> {
        let root = if let Ok(root) = std::env::var("FLUTTER_ROOT") {
            PathBuf::from(root)
        } else {
            let flutter = which::which("flutter").or(Err(Error::FlutterNotFound))?;
            let flutter = std::fs::canonicalize(flutter)?;
            flutter
                .parent()
                .ok_or(Error::FlutterNotFound)?
                .parent()
                .ok_or(Error::FlutterNotFound)?
                .to_owned()
        };
        Ok(Flutter { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn flutter(&self) -> Result<PathBuf, Error> {
        which::which("flutter").or(Err(Error::FlutterNotFound))
    }

    pub fn engine_version(&self) -> Result<String, Error> {
        let path = self
            .root
            .join("bin")
            .join("internal")
            .join("engine.version");
        Ok(std::fs::read_to_string(path).map(|v| v.trim().to_owned())?)
    }

    pub fn bundle(
        &self,
        root_dir: &Path,
        out_dir: &Path,
        build: Build,
        dart_main: &Path,
    ) -> Result<(), Error> {
        let flag = match build {
            Build::Debug => "--debug",
            Build::Release => "--release",
            Build::Profile => "--profile",
        };
        let status = Command::new(self.flutter()?)
            .current_dir(root_dir)
            .arg("build")
            .arg("bundle")
            .arg(flag)
            .arg("--track-widget-creation")
            .arg("--asset-dir")
            .arg(out_dir.join("flutter_assets"))
            .arg("--depfile")
            .arg(out_dir.join("snapshot_blob.bin.d"))
            .arg("--target")
            .arg(dart_main)
            .status()
            .expect("flutter build bundle");
        if status.code() != Some(0) {
            return Err(Error::FlutterError);
        }
        Ok(())
    }

    pub fn attach(&self, root_dir: &Path, debug_uri: &str) -> Result<(), Error> {
        let status = Command::new(self.flutter()?)
            .current_dir(root_dir)
            .arg("attach")
            .arg("--device-id=flutter-tester")
            .arg(format!("--debug-uri={}", debug_uri))
            .status()
            .expect("Success");
        if status.code() != Some(0) {
            return Err(Error::FlutterError);
        }
        Ok(())
    }

    pub fn aot(
        &self,
        root_dir: &Path,
        build_dir: &Path,
        host_engine: &Engine,
        target_engine: &Engine,
    ) -> Result<(), Error> {
        let host_engine_dir = host_engine.engine_dir();
        let target_engine_dir = target_engine.engine_dir();
        let snapshot = build_dir.join("kernel_snapshot.dill");

        let status = Command::new(host_engine.dart()?)
            .current_dir(root_dir)
            .arg(
                host_engine_dir
                    .join("gen")
                    .join("frontend_server.dart.snapshot"),
            )
            .arg("--sdk-root")
            .arg(host_engine_dir.join("flutter_patched_sdk"))
            .arg("--target=flutter")
            .arg("--aot")
            .arg("--tfa")
            .arg("-Ddart.vm.product=true")
            .arg("--packages")
            .arg(".packages")
            .arg("--output-dill")
            .arg(&snapshot)
            .arg(root_dir.join("lib").join("main.dart"))
            .status()
            .expect("Success");

        if status.code() != Some(0) {
            return Err(Error::FlutterError);
        }

        let gen_snapshot = [
            "gen_snapshot",
            "gen_snapshot_x64",
            "gen_snapshot_x86",
            "gen_snapshot_host_targeting_host",
            "gen_snapshot.exe",
        ]
        .iter()
        .map(|bin| target_engine_dir.join(bin))
        .find(|path| path.exists())
        .ok_or(Error::GenSnapshotNotFound)?;

        let status = Command::new(gen_snapshot)
            .current_dir(root_dir)
            .arg("--causal_async_stacks")
            .arg("--deterministic")
            .arg("--snapshot_kind=app-aot-elf")
            .arg("--strip")
            .arg(format!("--elf={}", build_dir.join("app.so").display()))
            .arg(&snapshot)
            .status()
            .expect("Success");

        if status.code() != Some(0) {
            return Err(Error::FlutterError);
        }

        Ok(())
    }

    pub fn drive(
        &self,
        host_engine: &Engine,
        root_dir: &Path,
        debug_uri: &str,
        dart_main: &Path,
    ) -> Result<(), Error> {
        let mut file = dart_main.file_stem().unwrap().to_owned();
        file.push("_test.dart");
        let driver = dart_main.parent().unwrap().join(file);

        // used by flutter_driver
        std::env::set_var("VM_SERVICE_URL", debug_uri);
        let status = Command::new(host_engine.dart()?)
            .current_dir(root_dir)
            .arg(driver)
            .status()
            .expect("Success");
        if status.code() != Some(0) {
            return Err(Error::FlutterError);
        }
        Ok(())
    }
}
