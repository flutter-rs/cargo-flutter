use crate::error::Error;
use curl::easy::Easy;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{mpsc, Mutex};
use std::{fs, thread};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Engine {
    version: String,
    target: String,
    build: Build,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Build {
    Debug,
    Release,
    Profile,
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
            "https://github.com/flutter-rs/engine-builds/releases/download/f-{0}-{1}/engine-{1}",
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

    pub fn engine_path(&self) -> PathBuf {
        let path = if let Ok(path) = std::env::var("FLUTTER_ENGINE_PATH") {
            PathBuf::from(path)
        } else {
            dirs::cache_dir()
                .expect("Cannot get cache dir")
                .join("flutter-engine")
                .join(&self.version)
                .join(&self.target)
                .join(self.build.build())
                .join(self.library_name())
        };
        path
    }

    pub fn download(&self, quiet: bool) {
        let url = self.download_url();
        let path = self.engine_path();
        let dir = path.parent().unwrap().to_owned();

        if path.exists() {
            return;
        }

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            // TODO: less unwrap, more error handling

            // Write the contents of rust-lang.org to stdout
            tx.send((0.0, 0.0)).unwrap();
            // create target dir

            fs::create_dir_all(&dir).unwrap();

            let download_file = dir.join("engine.zip");

            let mut file = File::create(&download_file).unwrap();

            let tx = Mutex::new(tx);

            let mut easy = Easy::new();

            println!("Starting download from {}", url);
            easy.url(&url).unwrap();
            easy.follow_location(true).unwrap();
            easy.progress(true).unwrap();
            easy.progress_function(move |total, done, _, _| {
                tx.lock().unwrap().send((total, done)).unwrap();
                true
            })
            .unwrap();
            easy.write_function(move |data| Ok(file.write(data).unwrap()))
                .unwrap();
            easy.perform().unwrap();

            println!("Download finished");

            println!("Extracting...");
            crate::unzip::unzip(&download_file, &dir).unwrap();
        });
        for (total, done) in rx.iter() {
            if !quiet {
                println!("Downloading flutter engine {} of {}", done, total);
            }
        }
    }

    pub fn latest_version() -> Result<String, Error> {
        let mut req =
            ureq::get("https://api.github.com/repos/flutter-rs/engine-builds/releases/latest");
        let tag_name = &req.call().into_json()?["tag_name"];
        let version = tag_name
            .as_str()
            .unwrap()
            .split('-')
            .nth(1)
            .unwrap()
            .to_string();
        Ok(version)
    }
}
