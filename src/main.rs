use cargo::core::compiler::{CompileMode, ProfileKind};
use cargo::ops::{CompileOptions, Packages};
use cargo::util::Config;
use cargo_flutter::package::apk::Apk;
use cargo_flutter::package::appimage::AppImage;
use cargo_flutter::{Build, Cargo, Engine, Error, Flutter, Item, Package, TomlConfig};
use clap::{App, AppSettings, Arg, SubCommand};
use exitfailure::ExitFailure;
use rand::Rng;
use std::{env, str};

fn main() -> Result<(), ExitFailure> {
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
                    Arg::with_name("quiet")
                        .long("quiet")
                        .help("avoids excessive printing to stdout"),
                )
                .arg(
                    Arg::with_name("no-flutter")
                        .long("no-flutter")
                        .help("shortcut for no-bundle, no-attach and no-aot"),
                )
                .arg(
                    Arg::with_name("no-bundle")
                        .long("no-bundle")
                        .help("Skips running flutter bundle"),
                )
                .arg(
                    Arg::with_name("no-attach")
                        .long("no-attach")
                        .help("Skips attaching the flutter debugger"),
                )
                .arg(
                    Arg::with_name("no-aot")
                        .long("no-aot")
                        .help("Skips creating aot blob"),
                )
                .arg(
                    Arg::with_name("format")
                        .short("f")
                        .long("format")
                        .value_name("FORMAT")
                        .takes_value(true)
                        .help("Packaging format"),
                )
                .arg(
                    Arg::with_name("sign")
                        .long("sign")
                        .help("Sign package in debug build"),
                )
                .arg(
                    Arg::with_name("no-sign")
                        .long("no-sign")
                        .help("Don't sign package in release build"),
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
        return Err(Error::NotCalledWithCargo.into());
    };

    let quiet = matches.is_present("quiet");
    let cargo_args: Vec<&str> = matches
        .values_of("cargo-args")
        .expect("cargo-args to not be null")
        .collect();
    let mut cargo_config = Config::default()?;
    let cargo = Cargo::new(&mut cargo_config, cargo_args)?;

    let build = if cargo.release() {
        Build::Release
    } else {
        Build::Debug
    };
    let aot = build == Build::Release;
    let sign = build == Build::Debug && matches.is_present("sign")
        || build == Build::Release && !matches.is_present("no-sign");
    let config = TomlConfig::load(&cargo).ok();
    let metadata = config
        .as_ref()
        .map(|config| config.metadata())
        .unwrap_or_default();
    let flutter = Flutter::new()?;
    let engine_version = metadata.engine_version().unwrap_or_else(|| {
        std::env::var("FLUTTER_ENGINE_VERSION")
            .ok()
            .unwrap_or_else(|| flutter.engine_version().unwrap())
    });

    log::debug!("FLUTTER_ROOT {}", flutter.root().display());
    log::debug!("FLUTTER_ENGINE_VERSION {}", engine_version);

    let triple = cargo.triple()?;
    let engine = Engine::new(engine_version.clone(), triple.clone(), build);
    let flutter_asset_dir = cargo.build_dir().join("flutter_assets");
    let snapshot_path = cargo.build_dir().join("app.so");
    let engine_path = cargo.build_dir().join("deps").join(engine.library_name());

    log::debug!("FLUTTER_ENGINE_PATH {}", engine.engine_path().display());
    log::debug!("FLUTTER_ASSET_DIR {}", flutter_asset_dir.display());

    engine.download(quiet)?;

    if !engine_path.exists() {
        std::fs::create_dir_all(engine_path.parent().unwrap())?;
        std::fs::copy(engine.engine_path(), &engine_path)?;

        if triple == "x86_64-pc-windows-msvc" {
            let from_dir = engine.engine_path().parent().unwrap().to_owned();
            let to_dir = engine_path.parent().unwrap();
            for file in &[
                "flutter_engine.dll.lib",
                "flutter_engine.dll.exp",
                "flutter_engine.dll.pdb",
            ] {
                std::fs::copy(from_dir.join(file), to_dir.join(file))?;
            }
        }
    }

    if config.is_some() {
        if !matches.is_present("no-flutter") && !matches.is_present("no-bundle") {
            println!("flutter build bundle");
            flutter.bundle(&cargo, build)?;
        }

        if !matches.is_present("no-flutter") && !matches.is_present("no-aot") {
            let host_triple = cargo.host_target()?;
            let host_engine = Engine::new(engine_version, host_triple, build);
            host_engine.download(quiet)?;

            if aot {
                flutter.aot(&cargo, &host_engine.engine_path(), &engine.engine_path())?;
            }
        }
    }

    match (cargo.cmd(), config) {
        ("build", Some(config)) => {
            let mut package = Package::new(&config.package.name);
            package.add_lib(engine_path);
            if aot {
                package.add_lib(snapshot_path);
            }
            package.add_asset(flutter_asset_dir);

            if !triple.contains("android") {
                cargo.exec()?;
                package.add_bin(cargo.build_dir().join(&config.package.name));

                if let Some(format) = matches.value_of("format") {
                    match format {
                        "appimage" => {
                            let builder = AppImage::new(metadata.appimage.unwrap_or_default());
                            builder.build(&cargo, &package, sign)?;
                        }
                        "dmg" => {}
                        "lipo" => {}
                        "nsis" => {}
                        _ => return Err(Error::FormatNotSupported.into()),
                    }
                }
            } else {
                use lib_cargo_apk::config::AndroidBuildTarget;
                let mut android_config = lib_cargo_apk::config::load(cargo.package()?).unwrap();
                let target = match triple.as_str() {
                    "armv7-linux-androideabi" => AndroidBuildTarget::ArmV7a,
                    "aarch64-linux-android" => AndroidBuildTarget::Arm64V8a,
                    "i686-linux-android" => AndroidBuildTarget::X86,
                    "x86_64-linux-android" => AndroidBuildTarget::X86_64,
                    _ => panic!("unsupported android target"),
                };
                android_config.build_targets = vec![target];
                android_config.release = build != Build::Debug;

                let mut options =
                    CompileOptions::new(cargo.workspace().config(), CompileMode::Build)?;
                options.build_config.profile_kind = if build == Build::Debug {
                    ProfileKind::Dev
                } else {
                    ProfileKind::Release
                };
                options.spec = if let Ok(package) = cargo.package() {
                    Packages::Packages(vec![package.name().to_string()])
                } else {
                    Packages::Default
                };

                let libs = lib_cargo_apk::build_shared_libraries(
                    cargo.workspace(),
                    &android_config,
                    options,
                    &cargo.build_dir(),
                )?;
                for (_, libs) in libs.shared_libraries.iter_all() {
                    for lib in libs {
                        package.add_lib(Item::new(lib.path.clone(), lib.filename.clone()));
                    }
                }
                if let Some(format) = matches.value_of("format") {
                    if format != "apk" {
                        return Err(Error::FormatNotSupported.into());
                    }
                    let builder = Apk::new(android_config);
                    builder.build(&cargo, &package, sign)?;
                }
            }
        }
        ("run", Some(_config)) => {
            let mut rng = rand::thread_rng();
            let port = rng.gen_range(1024, 49152);
            std::env::set_var("FLUTTER_AOT_SNAPSHOT", &snapshot_path);
            std::env::set_var("FLUTTER_ASSET_DIR", &flutter_asset_dir);
            std::env::set_var("DART_OBSERVATORY_PORT", port.to_string());
            cargo.spawn()?;

            if !matches.is_present("no-flutter") && !matches.is_present("no-attach") {
                flutter.attach(&cargo, &format!("http://127.0.0.1:{}", port))?;
            }
        }
        _ => cargo.exec()?,
    }

    Ok(())
}
