use android_build_tools::error::NdkError;
use cargo_subcommand::Error as SubcommandError;
use curl::Error as CurlError;
use failure::Error as Failure;
use std::fmt::{Display, Formatter, Result};
use std::io::Error as IoError;
use std::str::Utf8Error;
use toml::de::Error as TomlError;
use which::Error as WhichError;

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
    Which(WhichError),
    Io(IoError),
    Toml(TomlError),
    Utf8(Utf8Error),
    Curl(CurlError),
    Err(Failure),
    Ndk(NdkError),
    Subcommand(SubcommandError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
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

- [0] https://github.com/flutter-rs/engine-builds/releases"#,
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
            Error::Curl(error) => error.fmt(f),
            Error::Err(error) => error.fmt(f),
            Error::Ndk(error) => error.fmt(f),
            Error::Subcommand(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<WhichError> for Error {
    fn from(error: WhichError) -> Self {
        Error::Which(error)
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl From<TomlError> for Error {
    fn from(error: TomlError) -> Self {
        Error::Toml(error)
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::Utf8(error)
    }
}

impl From<CurlError> for Error {
    fn from(error: CurlError) -> Self {
        Error::Curl(error)
    }
}

impl From<Failure> for Error {
    fn from(error: Failure) -> Self {
        Error::Err(error)
    }
}

impl From<NdkError> for Error {
    fn from(error: NdkError) -> Self {
        Error::Ndk(error)
    }
}

impl From<SubcommandError> for Error {
    fn from(error: SubcommandError) -> Self {
        Error::Subcommand(error)
    }
}
