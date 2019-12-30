use cargo::util::Config;
use cargo_flutter::{Cargo, Engine, Flutter, TomlConfig};
use clap::{App, AppSettings, Arg, SubCommand};
use std::process::exit;
use std::{env, str};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let cargo_args: Vec<&str> = matches
        .values_of("cargo-args")
        .expect("cargo-args to not be null")
        .collect();

    let cargo_config = Config::default()?;
    let cargo = Cargo::new(&cargo_config, cargo_args)?;
    let _config = TomlConfig::load(&cargo)?;
    let flutter = Flutter::new()?;
    let engine = Engine::new(flutter.engine_version()?, cargo.triple()?);
    engine.download();

    if cargo.cmd() == "run" {
        println!("flutter build bundle");
        flutter.bundle(cargo.workspace());
    }

    let status = cargo.run(&engine.engine_path());

    /*if cargo.cmd() == "run" {
        flutter.attach(cargo.workspace(), "");
    }*/

    exit(status.code().unwrap_or(-1));
}
