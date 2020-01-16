#[derive(Debug)]
pub enum Error {
    PackageNotMember,
    EngineNotFound(String),
    FlutterNotFound,
    DartNotFound,
    GenSnapshotNotFound,
    FormatNotSupported,
    CargoError,
    FlutterError,
    NotCalledWithCargo,
    Which(which::Error),
    Io(std::io::Error),
    Toml(toml::de::Error),
    Utf8(std::str::Utf8Error),
    Err(failure::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::PackageNotMember => write!(f, "Package is not a member of the workspace"),
            Error::FlutterNotFound => write!(f, "Couldn't find flutter sdk"),
            Error::EngineNotFound(version) => write!(
                f,
                r#"We couldn't find the requested engine version '{}'.
This means that your flutter version is too old or to new.

To update flutter run `flutter upgrade`. If the problem persists the engine
build has not completed yet. This means you need to manually supply the flutter
engine version through one of the following methods:

```bash
export FLUTTER_ENGINE_VERSION = "..."
```

`Cargo.toml`
```toml
[package.metadata.flutter]
engine_version = "..."
```

You'll find the available builds on our github releases page [0].

- [0] https://github.com/flutter-rs/engine-builds/releases
"#,
                version,
            ),
            Error::DartNotFound => write!(f, "Could't find dart"),
            Error::GenSnapshotNotFound => write!(f, "Couldn't find gen_snapshot"),
            Error::FormatNotSupported => write!(f, "Format not supported"),
            Error::CargoError => write!(f, "Cargo did not exit successfully"),
            Error::FlutterError => write!(f, "Flutter did not exit successfully"),
            Error::NotCalledWithCargo => {
                write!(f, "This binary may only be called via `cargo flutter`.")
            }
            Error::Which(error) => error.fmt(f),
            Error::Io(error) => error.fmt(f),
            Error::Toml(error) => error.fmt(f),
            Error::Utf8(error) => error.fmt(f),
            Error::Err(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<which::Error> for Error {
    fn from(error: which::Error) -> Self {
        Error::Which(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Error::Toml(error)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        Error::Utf8(error)
    }
}

impl From<failure::Error> for Error {
    fn from(error: failure::Error) -> Self {
        Error::Err(error)
    }
}
