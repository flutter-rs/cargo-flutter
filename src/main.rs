use cargo::core::Workspace;
use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::util::Config;
use clap::{App, AppSettings, Arg, SubCommand};
use failure::format_err;
use serde::Deserialize;
use std::{env, fs, str};
use std::error::Error;
use std::path::Path;
use std::process::{exit, Command, ExitStatus};

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
    let config = load_config(&workspace, None)?;

    let rustc = cargo_config.load_global_rustc(Some(&workspace))?;
    let triple = matches.value_of("target").unwrap_or(rustc.host.as_str());
    let cargo_args: Vec<&str> = matches
        .values_of("cargo-args")
        .expect("cargo-args to not be null")
        .collect();

    log::debug!("Target triple is {}", triple);
    log::debug!("Requested flutter version is {:?}", config.version);

    let status = run(&env::current_dir().unwrap(), triple, cargo_args);

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
            .ok_or_else(|| format_err!("package `{}` is not a member of the workspace", package.as_str()))?
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

fn run(dir: &Path, triple: &str, cargo_args: Vec<&str>) -> ExitStatus {
    Command::new("cargo")
        .current_dir(dir)
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
