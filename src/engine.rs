use curl::easy::Easy;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{mpsc, Mutex};
use std::{fs, thread};

#[derive(Debug)]
pub struct Engine {
    version: String,
    target: String,
}

impl Engine {
    pub fn new(version: String, target: String) -> Engine {
        Engine { version, target }
    }

    pub fn download_url(&self) -> String {
        let (platform, file) = match self.target.as_str() {
            "x86_64-unknown-linux-gnu" => ("linux-x64", "linux-x64-embedder"),
            "x86_64-apple-darwin" => ("darwin-x64", "FlutterEmbedder.framework.zip"),
            "x86_64-pc-windows-msvc" => ("windows-x64", "windows-x64-embedder.zip"),
            _ => panic!("unsupported platform"),
        };
        format!(
            "https://storage.googleapis.com/flutter_infra/flutter/{}/{}/{}",
            &self.version, platform, file
        )
    }

    pub fn library_name(&self) -> &'static str {
        match self.target.as_str() {
            "x86_64-unknown-linux-gnu" => "libflutter_engine.so",
            "x86_64-apple-darwin" => "FlutterEmbedder.framework",
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
                .join(self.library_name())
        };
        log::info!("FLUTTER_ENGINE_PATH {}", path.display());
        path
    }

    pub fn download(&self) {
        let url = self.download_url();
        let path = self.engine_path();
        let dir = path.parent().unwrap().to_owned();
        let is_macos = self.target.contains("apple");

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
            let zip_file = File::open(&download_file).unwrap();
            let reader = BufReader::new(zip_file);
            let unzipper = unzip::Unzipper::new(reader, &dir);
            unzipper.unzip().unwrap();

            // mac framework file is a double zip file
            if is_macos {
                Command::new("unzip")
                    .args(&[
                        "FlutterEmbedder.framework.zip",
                        "-d",
                        "FlutterEmbedder.framework",
                    ])
                    .current_dir(&dir)
                    .status()
                    .unwrap();

                // TODO: fixme
                // unzip bug! Extracted file corrupted!
                // let zip_file = File::open(dir.join("FlutterEmbedder.framework.zip")).unwrap();
                // let reader = BufReader::new(zip_file);
                // let unzipper = unzip::Unzipper::new(reader, dir.join("FlutterEmbedder.framework"));
                // unzipper.unzip().unwrap();
            }
        });
        for (total, done) in rx.iter() {
            println!("Downloading flutter engine {} of {}", done, total);
        }
    }
}
