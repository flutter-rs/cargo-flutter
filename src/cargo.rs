use crate::error::Error;
use cargo::core::Workspace;
use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::util::Config;
use std::path::Path;
use std::process::{Command, ExitStatus};

pub struct Cargo<'a> {
    args: Vec<&'a str>,
    workspace: Workspace<'a>,
}

impl<'a> Cargo<'a> {
    pub fn new(config: &'a Config, args: Vec<&'a str>) -> Result<Self, Error> {
        let root_manifest = find_root_manifest_for_wd(config.cwd())?;
        let workspace = Workspace::new(&root_manifest, config)?;
        Ok(Self { args, workspace })
    }

    fn arg<F: Fn(&str) -> bool>(&self, matches: F) -> Option<&str> {
        self.args
            .iter()
            .position(|f| matches(f))
            .map(|pos| self.args.iter().nth(pos + 1))
            .unwrap_or_default()
            .map(|v| *v)
    }

    pub fn cmd(&self) -> &str {
        self.args.iter().next().expect("Expected command")
    }

    pub fn target(&self) -> Option<&str> {
        self.arg(|f| f == "--target")
    }

    pub fn package(&self) -> Option<&str> {
        self.arg(|f| f == "--package" || f == "-p")
    }

    pub fn host_target(&self) -> Result<String, Error> {
        let rustc = self
            .workspace
            .config()
            .load_global_rustc(Some(&self.workspace))?;
        Ok(rustc.host.as_str().to_string())
    }

    pub fn triple(&self) -> Result<String, Error> {
        if let Some(target) = self.target() {
            Ok(target.to_string())
        } else {
            self.host_target()
        }
    }

    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    pub fn run(&self, engine_path: &Path) -> ExitStatus {
        let target_dir = self.workspace.target_dir().into_path_unlocked();
        let rustflags = format!(
            "-Clink-arg=-L{0} -Clink-arg=-Wl,-rpath={0}",
            engine_path.parent().unwrap().display(),
        );
        Command::new("cargo")
            .current_dir(self.workspace.config().cwd())
            .env("RUSTFLAGS", rustflags)
            .args(&self.args)
            .arg("--target-dir")
            .arg(target_dir.join("flutter"))
            .status()
            .expect("Success")
    }
}
