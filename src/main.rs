use std::{env, fs};
use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;
use clap::Parser;

#[derive(Debug, Deserialize)]
struct Config {
    files: Vec<PathBuf>,
    theme_name: String,
}

impl Config {
    fn from_path(path: PathBuf) -> Self {
        let slice = fs::read(path).expect("config file read failed");
        let config: serde_json::Result<Self> = serde_json::from_slice(&slice);
        config.expect("serde_json::from_slice failed")
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    config_path: Option<PathBuf>,

    #[arg(long)]
    theme_path: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let configtool_dir = if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        let mut path = PathBuf::new();
        path.push(config_home);
        path.push("configtool");
        path
    } else {
        let home = env::var("HOME").expect("$XDG_CONFIG_HOME or $HOME must be defined");
        let mut path = PathBuf::new();
        path.push(home);
        path.push(".config/configtool");
        path
    };

    let config = if let Some(config_path) = args.config_path {
        Config::from_path(config_path)
    } else {
        let mut path = configtool_dir.clone();
        path.push("config.json");
        Config::from_path(path)
    };

    
    let theme = if let Some(path) = args.theme_path {
        let slice = fs::read(path).expect("config file read failed");
        let config: serde_json::Result<HashMap<String, String>> = serde_json::from_slice(&slice);
        config.expect("serde_json::from_slice failed")
    } else {
        let mut path = configtool_dir.clone();
        path.push("themes");
        path.push(&config.theme_name);
        let slice = fs::read(path).expect("config file read failed");
        let config: serde_json::Result<HashMap<String, String>> = serde_json::from_slice(&slice);
        config.expect("serde_json::from_slice failed")
    };

    println!("{:?}", config);
    println!("{:?}", theme);

    for file in config.files {
        println!("{:?}", file);
        let mut contents = fs::read_to_string(&file).expect("File should exist");
        for (key, value) in &theme {
            contents = contents.replace(key, value);
        }
        fs::write(&file, &contents).expect("File write should work");
    }
}
