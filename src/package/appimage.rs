use crate::package::Package;
use cargo::core::Workspace;
use serde::Deserialize;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct TomlAppImage {
    name: Option<String>,
    icon: Option<String>,
}

pub struct AppImage {
    toml: TomlAppImage,
}

impl AppImage {
    pub fn new(toml: TomlAppImage) -> Self {
        Self { toml }
    }

    pub fn build(
        &self,
        workspace: &Workspace,
        package: &Package,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let target_dir = workspace.target_dir().into_path_unlocked();
        let appimage_dir = target_dir.join("appimage");
        let name = self.toml.name.as_ref().unwrap_or(&package.name);
        let exec = &package.name;
        let icon_path = self
            .toml
            .icon
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| workspace.root().join("assets").join("icon.svg"));
        if !icon_path.exists() {
            return Err(failure::format_err!("Icon not found {}", icon_path.display()).into());
        }
        let icon = icon_path
            .file_stem()
            .map(|f| f.to_str().unwrap())
            .unwrap_or("icon")
            .to_string();
        std::fs::remove_dir_all(&appimage_dir)?;

        let bin_dir = appimage_dir.join("usr").join("bin");
        std::fs::create_dir_all(&bin_dir)?;
        for bin in &package.bin {
            std::fs::copy(bin, bin_dir.join(bin.file_name().unwrap()))?;
        }

        let lib_dir = appimage_dir.join("usr").join("lib");
        std::fs::create_dir_all(&lib_dir)?;
        for lib in &package.lib {
            std::fs::copy(lib, lib_dir.join(lib.file_name().unwrap()))?;
        }

        let asset_dir = appimage_dir.join("usr").join("share");
        std::fs::create_dir_all(&asset_dir)?;
        for asset in &package.asset {
            copy_dir::copy_dir(asset, asset_dir.join(asset.file_name().unwrap()))?;
        }

        let apprun = appimage_dir.join("AppRun");
        std::fs::write(&apprun, APP_RUN)?;
        std::fs::set_permissions(&apprun, Permissions::from_mode(0o755))?;

        let desktop = appimage_dir.join(format!("{}.desktop", exec));
        std::fs::write(&desktop, gen_desktop(name, exec, &icon))?;
        std::fs::set_permissions(&desktop, Permissions::from_mode(0o755))?;

        std::fs::copy(
            &icon_path,
            appimage_dir.join(icon_path.file_name().unwrap()),
        )?;

        Command::new("appimagetool")
            .current_dir(&target_dir)
            .arg("appimage")
            .status()
            .expect("Success");

        Ok(())
    }
}

const APP_RUN: &str = r#"#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin/${PATH:+:$PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib/:${LD_LIBRARY_PATH:+:$LDLIBRARY_PATH}"
EXEC=$(grep -e '^Exec=.*' "${HERE}"/*.desktop | head -n 1 | cut -d "=" -f 2 | cut -d " " -f 1)
exec "${EXEC}" "$@"
"#;

fn gen_desktop(name: &str, exec: &str, icon: &str) -> String {
    format!(
        r#"[Desktop Entry]
Name={}
Exec={}
Icon={}
Type=Application
Categories=Utility;
"#,
        name, exec, icon
    )
}
