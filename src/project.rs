use crate::config::Config;
use crate::engine::{Build, Engine};
use crate::error::Error;
use crate::flutter::Flutter;
use cargo_subcommand::{Profile, Subcommand};
use std::path::PathBuf;

pub struct Project {
    pub flutter: Flutter,
    pub config: Config,
    pub build: Build,
    pub root_dir: PathBuf,
    pub out_dir: PathBuf,
    pub host_triple: String,
    pub target_triple: String,
    pub host_engine: Engine,
    pub target_engine: Engine,
    pub flutter_assets: PathBuf,
    pub snapshot: PathBuf,
    pub dart_main: PathBuf,
    pub target_engine_path: PathBuf,
}

impl Project {
    pub fn from_subcommand(cmd: &Subcommand, dart_main: Option<PathBuf>) -> Result<Self, Error> {
        let flutter = Flutter::new()?;
        let config = Config::parse_from_toml(cmd.manifest())?;
        let engine_version = config.engine_version.clone().unwrap_or_else(|| {
            std::env::var("FLUTTER_ENGINE_VERSION")
                .ok()
                .unwrap_or_else(|| flutter.engine_version().unwrap())
        });
        let build = match cmd.profile() {
            Profile::Release => Build::Release,
            _ => Build::Debug,
        };

        // Download host engine
        let host_triple = cmd.host_triple().to_string();
        let host_engine = Engine::new(engine_version.clone(), host_triple.clone(), build);
        host_engine.download(cmd.quiet())?;

        // Download target engine
        let target_triple = cmd.target().unwrap_or(&host_triple).to_string();
        let target_engine = Engine::new(engine_version, target_triple.clone(), build);
        target_engine.download(cmd.quiet())?;

        let root_dir = cmd.manifest().parent().unwrap().to_path_buf();
        let out_dir = cmd
            .target_dir()
            .join(cmd.target().unwrap_or(""))
            .join(cmd.profile());

        let flutter_assets = out_dir.join("flutter_assets");
        let snapshot = out_dir.join("app.so");
        let dart_main = dart_main.unwrap_or_else(|| PathBuf::from("lib/main.dart"));
        let target_engine_path = out_dir.join("deps").join(target_engine.library_name());
        Ok(Self {
            flutter,
            config,
            build,
            root_dir,
            out_dir,
            host_triple,
            target_triple,
            host_engine,
            target_engine,
            flutter_assets,
            snapshot,
            dart_main,
            target_engine_path,
        })
    }

    pub fn copy_engine(&self) -> Result<(), Error> {
        let src = self.target_engine.engine_path();
        let src_dir = src.parent().unwrap();
        let dst = &self.target_engine_path;
        let dst_dir = self.target_engine_path.parent().unwrap();
        std::fs::create_dir_all(src_dir)?;
        std::fs::copy(&src, &dst)?;

        if self.target_triple.contains("windows") {
            let file = "flutter_engine.lib";
            std::fs::copy(src_dir.join(file), dst_dir.join(file))?;
        }
        Ok(())
    }

    pub fn bundle(&self) -> Result<(), Error> {
        self.flutter
            .bundle(&self.root_dir, &self.out_dir, self.build, &self.dart_main)
    }

    pub fn aot(&self) -> Result<(), Error> {
        self.flutter.aot(
            &self.root_dir,
            &self.out_dir,
            &self.host_engine,
            &self.target_engine,
        )
    }

    pub fn drive(&self, observatory: &str) -> Result<(), Error> {
        self.flutter.drive(
            &self.host_engine,
            &self.root_dir,
            observatory,
            &self.dart_main,
        )
    }

    pub fn attach(&self, observatory: &str) -> Result<(), Error> {
        self.flutter.attach(&self.root_dir, observatory)
    }
}
