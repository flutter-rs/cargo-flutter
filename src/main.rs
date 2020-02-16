use android_build_tools::cargo::cargo_build;
use android_build_tools::ndk::Ndk;
use android_build_tools::target::Target;
use cargo_flutter::package::apk::Apk;
use cargo_flutter::package::appimage::AppImage;
use cargo_flutter::{Build, Error, Package, Project};
use cargo_subcommand::{Profile, Subcommand};
use exitfailure::ExitDisplay;
use rand::Rng;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug, Default)]
struct Args {
    flutter: Option<bool>,
    bundle: Option<bool>,
    attach: Option<bool>,
    aot: Option<bool>,
    dart_main: Option<PathBuf>,
    drive: bool,
    format: Option<String>,
    sign: Option<bool>,
}

fn main() -> Result<(), ExitDisplay<Error>> {
    Ok(cli_main()?)
}

fn cli_main() -> Result<(), Error> {
    env_logger::init();

    let mut args = Args::default();
    let cmd = Subcommand::new("flutter", |name, value| {
        let mut matched = true;
        match (name, value) {
            ("--no-flutter", None) => args.flutter = Some(false),
            ("--no-bundle", None) => args.bundle = Some(false),
            ("--no-aot", None) => args.aot = Some(false),
            ("--no-attach", None) => args.attach = Some(false),
            ("--dart-main", value) => args.dart_main = value.map(|s| PathBuf::from(s)),
            ("--drive", None) => args.drive = true,
            ("--format", value) => args.format = value.map(|s| s.to_string()),
            ("--sign", None) => args.sign = Some(true),
            ("--no-sign", None) => args.sign = Some(false),
            _ => matched = false,
        }
        Ok(matched)
    })?;

    let project = Project::from_subcommand(&cmd, args.dart_main.take())?;

    match cmd.cmd() {
        "build" => {
            build(&cmd, &args, &project)?;
        }
        "run" => {
            let package = build(&cmd, &args, &project)?;

            if package.triple().contains("android") {
                // TODO
            } else {
                let mut rng = rand::thread_rng();
                let port = rng.gen_range(1024, 49152);
                let observatory = format!("http://127.0.0.1:{}", port);
                std::env::set_var("FLUTTER_AOT_SNAPSHOT", &project.snapshot);
                std::env::set_var("FLUTTER_ASSET_DIR", &project.flutter_assets);
                std::env::set_var("DART_OBSERVATORY_PORT", port.to_string());

                if args.drive {
                    project.drive(&observatory)?;
                } else if args.flutter.unwrap_or(true) && args.attach.unwrap_or(true) {
                    project.attach(&observatory)?;
                }
            }
        }
        _ => {
            Command::new("cargo")
                .arg(cmd.cmd())
                .args(cmd.args())
                .status()?;
        }
    }

    Ok(())
}

fn build(cmd: &Subcommand, args: &Args, project: &Project) -> Result<Package, Error> {
    // Copy target engine to deps dir
    if !project.target_engine_path.exists() {
        project.copy_engine()?;

        // On android create an apk for the flutter engine to speed up build/run time.
        if project.target_triple.contains("android") {
            let mut package = Package::from_project(&project);
            package.add_lib(&project.target_engine_path);
            let target = Target::from_rust_triple(&project.target_triple)? as u8;
            let build = project.build as u8;
            Apk::new(project.config.apk.clone(), target * build, Some("engine")).build(&package)?;
        }
    }

    // Build flutter_assets
    if args.flutter.unwrap_or(true) && args.bundle.unwrap_or(true) {
        println!("flutter build bundle {:?}", &project.dart_main);
        project.bundle()?;
    }

    // Build aot binary
    if args.flutter.unwrap_or(true) && args.aot.unwrap_or(*cmd.profile() == Profile::Release) {
        project.aot()?;
    }

    // Build rust
    let mut cargo = if project.target_triple.contains("android") {
        let ndk = Ndk::from_env()?;
        let target = Target::from_rust_triple(&project.target_triple)?;
        cargo_build(&ndk, target, ndk.default_platform())?
    } else {
        let mut cargo = Command::new("cargo");
        cargo.arg("build");
        cargo
    };
    cargo.args(cmd.args()).status()?;

    // Package result
    let mut package = Package::from_project(&project);
    if project.build == Build::Release {
        package.add_lib(&project.snapshot);
    }
    package.add_asset(&project.flutter_assets);

    if project.target_triple.contains("android") {
        let lib = format!("lib{}.so", cmd.crate_name().replace("-", "_"));
        package.add_lib(project.out_dir.join(lib));
    } else {
        package.add_lib(&project.target_engine_path);
        package.add_bin(project.out_dir.join(cmd.crate_name()));
    }

    let sign = args
        .sign
        .unwrap_or_else(|| *cmd.profile() == Profile::Release);

    match args.format.as_ref().map(|s| &**s) {
        Some("appimage") => {
            let builder = AppImage::new(project.config.appimage.clone(), sign);
            builder.build(&package)?;
        }
        Some("apk") => {
            let target = Target::from_rust_triple(&project.target_triple)? as u8;
            let build = project.build as u8;
            let builder = Apk::new(project.config.apk.clone(), target * build, None);
            builder.build(&package)?;
        }
        Some("dmg") => {}
        Some("lipo") => {}
        Some("nsis") => {}
        None => {}
        _ => return Err(Error::FormatNotSupported.into()),
    }

    Ok(package)
}
