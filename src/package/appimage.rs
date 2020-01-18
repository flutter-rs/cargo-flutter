use crate::cargo::Cargo;
use crate::package::Package;
use failure::Error;
use serde::Deserialize;
use std::fs::Permissions;
#[cfg(unix)]
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

    #[cfg(not(unix))]
    pub fn build(&self, cargo: &Cargo, package: &Package, sign: bool) -> Result<(), Error> {
        Err(failure::format_err!("Creating appimages only supported from a unix host.").into())
    }

    #[cfg(unix)]
    pub fn build(&self, cargo: &Cargo, package: &Package, sign: bool) -> Result<(), Error> {
        let build_dir = cargo.build_dir();
        let appimage_dir = build_dir.join("appimage");
        let name = self.toml.name.as_ref().unwrap_or(&package.name);
        let exec = &package.name;
        let icon_path = self
            .toml
            .icon
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| cargo.workspace().root().join("assets").join("icon.svg"));
        if !icon_path.exists() {
            return Err(failure::format_err!("Icon not found {}", icon_path.display()));
        }
        let icon = icon_path
            .file_stem()
            .map(|f| f.to_str().unwrap())
            .unwrap_or("icon")
            .to_string();
        std::fs::remove_dir_all(&appimage_dir).ok();

        let bin_dir = appimage_dir.join("usr").join("bin");
        std::fs::create_dir_all(&bin_dir)?;
        for bin in package.bins() {
            std::fs::copy(bin.path(), bin_dir.join(bin.name()))?;
        }

        let lib_dir = appimage_dir.join("usr").join("lib");
        std::fs::create_dir_all(&lib_dir)?;
        for lib in package.libs() {
            std::fs::copy(lib.path(), lib_dir.join(lib.name()))?;
        }

        let asset_dir = appimage_dir.join("usr").join("share");
        std::fs::create_dir_all(&asset_dir)?;
        for asset in package.assets() {
            copy_dir::copy_dir(asset.path(), asset_dir.join(asset.name()))?;
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

        let appimagetool = which::which("appimagetool")
            .or(Err(failure::format_err!("appimagetool not found")))?;
        let mut cmd = Command::new(appimagetool);
        cmd.current_dir(&build_dir).arg("appimage");
        if sign {
            cmd.arg("--sign");
        }
        cmd.status().expect("Success");

        Ok(())
    }
}

const APP_RUN: &str = r#"#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin/${PATH:+:$PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib/:${LD_LIBRARY_PATH:+:$LDLIBRARY_PATH}"
export FLUTTER_ASSET_DIR="${HERE}/usr/share/flutter_assets"
export FLUTTER_AOT_SNAPSHOT="${HERE}/usr/lib/app.so"
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
