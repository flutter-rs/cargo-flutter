use crate::error::Error;
use cargo::core::Workspace;
use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::util::Config;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

    pub fn release(&self) -> bool {
        self.args.iter().find(|f| **f == "--release").is_some()
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

    pub fn target_dir(&self) -> PathBuf {
        self.workspace().target_dir().into_path_unlocked()
    }

    pub fn flutter_dir(&self) -> PathBuf {
        self.target_dir().join("flutter")
    }

    pub fn build_dir(&self) -> PathBuf {
        let flutter_dir = self.flutter_dir();
        let triple_dir = if let Some(target) = self.target() {
            flutter_dir.join(target)
        } else {
            flutter_dir
        };
        if self.release() {
            triple_dir.join("release")
        } else {
            triple_dir.join("debug")
        }
    }

    fn cargo_command(&self, engine_path: &Path) -> Command {
        let engine_dir = engine_path.parent().unwrap();
        let rpath = if !self.release() {
            format!(" -Clink-arg=-Wl,-rpath={}", engine_dir.display())
        } else {
            "".to_string()
        };
        let rustflags = format!("-Clink-arg=-L{}{}", engine_dir.display(), rpath);
        let mut cmd = Command::new("cargo");
        cmd.current_dir(self.workspace.config().cwd())
            .env("RUSTFLAGS", rustflags)
            .args(&self.args)
            .arg("--target-dir")
            .arg(self.flutter_dir());
        cmd
    }

    pub fn build(&self, engine_path: &Path) -> Result<(), Error> {
        let status = self.cargo_command(engine_path).status().expect("Success");
        if status.code() != Some(0) {
            return Err(Error::CargoError);
        }
        Ok(())
    }

    pub fn run(&self, engine_path: &Path) -> Result<String, Error> {
        let mut child = self.cargo_command(engine_path)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Success");
        let stdout = child.stdout.as_mut().unwrap();
        let mut buffer = [0; 70];
        stdout.read_exact(&mut buffer)?;
        let string = std::str::from_utf8(&buffer)?;
        Ok(string[34..].to_string())
    }
}
