use cargo::core::Workspace;
use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::util::Config;
use cargo_flutter::EngineInfo;
use clap::{App, AppSettings, Arg, SubCommand};
use failure::format_err;
use serde::Deserialize;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::{exit, Command, ExitStatus};
use std::{env, fs, str};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let app_matches = App::new("cargo-flutter")
        .bin_name("cargo")
        .subcommand(
            SubCommand::with_name("flutter")
                .setting(AppSettings::TrailingVarArg)
                .version(env!("CARGO_PKG_VERSION"))
                .author("flutter-rs")
                .about("Provides a smooth experience for developing flutter-rs apps.")
                .arg(
                    Arg::with_name("target")
                        .long("target")
                        .value_name("TARGET")
                        .takes_value(true)
                        .help("The triple for the target"),
                )
                .arg(
                    Arg::with_name("cargo-args")
                        .value_name("CARGO_ARGS")
                        .takes_value(true)
                        .required(true)
                        .multiple(true),
                ),
        )
        .get_matches();

    let matches = if let Some(matches) = app_matches.subcommand_matches("flutter") {
        matches
    } else {
        eprintln!("This binary may only be called via `cargo flutter`.");
        exit(1);
    };

    let cargo_config = Config::default()?;
    let root_manifest = find_root_manifest_for_wd(cargo_config.cwd())?;
    let workspace = Workspace::new(&root_manifest, &cargo_config)?;
    // TODO -p flag
    let config = load_config(&workspace, None)?;

    let rustc = cargo_config.load_global_rustc(Some(&workspace))?;
    let target = matches.value_of("target").unwrap_or(rustc.host.as_str());
    let cargo_args: Vec<&str> = matches
        .values_of("cargo-args")
        .expect("cargo-args to not be null")
        .collect();

    let engine_path = download(config.version, target.to_string())?;

    let status = run(cargo_config.cwd(), target, cargo_args, &engine_path);

    exit(status.code().unwrap_or(-1));
}

fn load_config(
    workspace: &Workspace,
    flag_package: Option<String>,
) -> Result<TomlFlutter, Box<dyn Error>> {
    let package = if let Some(package) = flag_package {
        workspace
            .members()
            .find(|pkg| pkg.name().as_str() == package.as_str())
            .ok_or_else(|| {
                format_err!(
                    "package `{}` is not a member of the workspace",
                    package.as_str()
                )
            })?
    } else {
        workspace.current()?
    };

    let bytes = fs::read(package.manifest_path())?;
    let string = str::from_utf8(&bytes)?;
    let config: TomlConfig = toml::from_str(string)?;
    let flutter = config
        .package
        .metadata
        .unwrap_or_default()
        .flutter
        .unwrap_or_default();
    Ok(flutter)
}

fn download(_version: Option<String>, target: String) -> Result<PathBuf, Box<dyn Error>> {
    let version = cargo_flutter::get_flutter_version()?;
    log::debug!("Engine version is {:?}", version);

    let info = EngineInfo::new(version, target);
    println!("Checking flutter engine status");

    if let Ok(tx) = info.download() {
        for (total, done) in tx.iter() {
            println!("Downloading flutter engine {} of {}", done, total);
        }
    }

    let engine_path = info.engine_path();
    log::debug!("Engine path is {:?}", engine_path);
    Ok(engine_path)
}

fn run(dir: &Path, triple: &str, cargo_args: Vec<&str>, engine_path: &Path) -> ExitStatus {
    Command::new("cargo")
        .current_dir(dir)
        .env("RUSTFLAGS", format!("-Clink-arg=-L{}", engine_path.parent().unwrap().display()))
        .args(cargo_args)
        .arg("--target")
        .arg(&triple)
        .status()
        .expect("Success")
}

#[derive(Debug, Clone, Deserialize)]
struct TomlConfig {
    package: TomlPackage,
}

#[derive(Debug, Clone, Deserialize)]
struct TomlPackage {
    name: String,
    metadata: Option<TomlMetadata>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct TomlMetadata {
    flutter: Option<TomlFlutter>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct TomlFlutter {
    version: Option<String>,
}
