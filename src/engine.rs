use crate::error::Error;
use curl::easy::Easy;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Engine {
    version: String,
    target: String,
    build: Build,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Build {
    Debug = 1,
    Profile = 2,
    Release = 3,
}

impl Build {
    pub fn build(&self) -> &str {
        match self {
            Self::Debug => "debug_unopt",
            Self::Release => "release",
            Self::Profile => "profile",
        }
    }
}

impl Engine {
    pub fn new(version: String, target: String, build: Build) -> Engine {
        Engine {
            version,
            target,
            build,
        }
    }

    pub fn download_url(&self) -> String {
        let build = self.build.build();
        let platform = match self.target.as_str() {
            "x86_64-unknown-linux-gnu" => format!("linux_x64-host_{}", build),
            "armv7-linux-androideabi" => format!("linux_x64-android_{}", build),
            "aarch64-linux-android" => format!("linux_x64-android_{}_arm64", build),
            "i686-linux-android" => format!("linux_x64-android_{}_x64", build),
            "x86_64-linux-android" => format!("linux_x64-android_{}_x86", build),
            "x86_64-apple-darwin" => format!("macosx_x64-host_{}", build),
            "armv7-apple-ios" => format!("macosx_x64-ios_{}_arm", build),
            "aarch64-apple-ios" => format!("macosx_x64-ios_{}", build),
            "x86_64-pc-windows-msvc" => format!("windows_x64-host_{}", build),
            _ => panic!("unsupported platform"),
        };
        format!(
            "https://github.com/flutter-rs/engine-builds/releases/download/f-{0}/{1}.zip",
            &self.version, platform
        )
    }

    pub fn library_name(&self) -> &'static str {
        match self.target.as_str() {
            "x86_64-unknown-linux-gnu" => "libflutter_engine.so",
            "armv7-linux-androideabi" => "libflutter_engine.so",
            "aarch64-linux-android" => "libflutter_engine.so",
            "i686-linux-android" => "libflutter_engine.so",
            "x86_64-linux-android" => "libflutter_engine.so",
            "x86_64-apple-darwin" => "libflutter_engine.dylib",
            "armv7-apple-ios" => "libflutter_engine.dylib",
            "aarch64-apple-ios" => "libflutter_engine.dylib",
            "x86_64-pc-windows-msvc" => "flutter_engine.dll",
            _ => panic!("unsupported platform"),
        }
    }

    pub fn engine_dir(&self) -> PathBuf {
        dirs::cache_dir()
            .expect("Cannot get cache dir")
            .join("flutter-engine")
            .join(&self.version)
            .join(&self.target)
            .join(self.build.build())
    }

    pub fn engine_path(&self) -> PathBuf {
        self.engine_dir().join(self.library_name())
    }

    pub fn download(&self, quiet: bool) -> Result<(), Error> {
        let url = self.download_url();
        let path = self.engine_path();
        let dir = path.parent().unwrap().to_owned();

        if path.exists() {
            return Ok(());
        }

        std::fs::create_dir_all(&dir)?;

        println!("Starting download from {}", url);
        let download_file = dir.join("engine.zip");
        let mut file = File::create(&download_file)?;
        let mut last_done = 0.0;

        let mut easy = Easy::new();
        easy.fail_on_error(true)?;
        easy.url(&url)?;
        easy.follow_location(true)?;
        easy.progress(true)?;
        easy.progress_function(move |total, done, _, _| {
            if done > last_done {
                last_done = done;
                if !quiet {
                    println!("Downloading flutter engine {} of {}", done, total);
                }
            }
            true
        })?;
        easy.write_function(move |data| Ok(file.write(data).unwrap()))?;

        easy.perform()
            .or_else(|_| Err(Error::EngineNotFound(self.version.clone())))?;
        println!("Download finished");

        println!("Extracting...");
        crate::unzip::unzip(&download_file, &dir)?;

        Ok(())
    }

    pub fn dart(&self) -> Result<PathBuf, Error> {
        let host_engine_dir = self.engine_dir();
        ["dart", "dart.exe"]
            .iter()
            .map(|bin| host_engine_dir.join(bin))
            .find(|path| path.exists())
            .ok_or(Error::DartNotFound)
    }
}
