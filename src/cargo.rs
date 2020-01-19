use crate::error::Error;
use cargo::core::{Package, Workspace};
use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::util::Config;
use std::path::PathBuf;
use std::process::Command;

pub struct Cargo<'a> {
    args: Vec<&'a str>,
    workspace: Workspace<'a>,
}

impl<'a> Cargo<'a> {
    pub fn new(config: &'a mut Config, args: Vec<&'a str>) -> Result<Self, Error> {
        let root_manifest = find_root_manifest_for_wd(config.cwd())?;
        let target_dir = root_manifest
            .parent()
            .unwrap()
            .join("target")
            .join("flutter");
        config
            .configure(0, None, &None, false, false, false, &Some(target_dir), &[])
            .unwrap();

        let workspace = Workspace::new(&root_manifest, config)?;
        Ok(Self { args, workspace })
    }

    fn arg<F: Fn(&str) -> bool>(&self, matches: F) -> Option<&str> {
        self.args
            .iter()
            .position(|f| matches(f))
            .map(|pos| self.args.get(pos + 1))
            .unwrap_or_default()
            .cloned()
    }

    pub fn cmd(&self) -> &str {
        self.args.iter().next().expect("Expected command")
    }

    pub fn target(&self) -> Option<&str> {
        self.arg(|f| f == "--target")
    }

    pub fn package(&self) -> Result<&Package, Error> {
        Ok(
            if let Some(package) = self.arg(|f| f == "--package" || f == "-p") {
                self.workspace()
                    .members()
                    .find(|pkg| pkg.name().as_str() == package)
                    .ok_or(Error::PackageNotMember)?
            } else {
                self.workspace().current()?
            },
        )
    }

    pub fn release(&self) -> bool {
        self.args.iter().any(|f| *f == "--release")
    }

    pub fn host_triple(&self) -> Result<String, Error> {
        let rustc = self
            .workspace
            .config()
            .load_global_rustc(Some(&self.workspace))?;
        Ok(rustc.host.as_str().to_string())
    }

    pub fn target_triple(&self) -> Result<String, Error> {
        if let Some(target) = self.target() {
            Ok(target.to_string())
        } else {
            self.host_triple()
        }
    }

    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    pub fn target_dir(&self) -> PathBuf {
        self.workspace().target_dir().into_path_unlocked()
    }

    pub fn build_dir(&self) -> PathBuf {
        let flutter_dir = self.target_dir();
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

    fn cargo_command(&self) -> Command {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(self.workspace.config().cwd())
            .args(&self.args)
            .arg("--target-dir")
            .arg(self.target_dir());
        cmd
    }

    pub fn exec(&self) -> Result<(), Error> {
        let status = self.cargo_command().status().expect("Success");
        if status.code() != Some(0) {
            return Err(Error::CargoError);
        }
        Ok(())
    }

    pub fn spawn(&self) -> Result<(), Error> {
        self.cargo_command().spawn().expect("Success");
        Ok(())
    }
}
