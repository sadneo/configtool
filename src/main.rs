use std::env;
use std::fs::{self, File};
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use clap::Parser;

const DEFAULT_CONFIG: Config = Config {
    files: vec![],
    theme_name: String::new(),
};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    files: Vec<PathBuf>,
    theme_name: String,
}

// manual theme doesn't exist
// theme not specified in config
// theme specified in config, but doesn't exist in files/themes

impl Config {
    fn from_path(path: &PathBuf) -> Option<Self> {
        let slice = fs::read(path).ok()?;
        let config: serde_json::Result<Self> = serde_json::from_slice(&slice);
        Some(config.expect("serde_json::from_slice failed"))
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
        let Some(config) = Config::from_path(&config_path) else {
            eprintln!("manually specified config path doesn't exist");
            return;
        };
        config
    } else {
        let mut path = configtool_dir.clone();
        path.push("config.json");
        match Config::from_path(&path) {
            Some(config) => config,
            None => {
                if let Err(e) = fs::create_dir_all(&path) {
                    eprintln!("error creating config directory: {:?}", e);
                    return;
                }
                let file = match File::create(&path) {
                    Ok(file) => file,
                    Err(e) => {
                        eprintln!("failed to create config file: {:?}", e);
                        return;
                    }
                };
                path.pop();
                path.push("themes");
                if let Err(e) = fs::create_dir_all(&path) {
                    eprintln!("error creating themes directory: {:?}", e);
                    return;
                }
                if let Err(e) = serde_json::to_writer_pretty(file, &DEFAULT_CONFIG) {
                    eprintln!("error parsing config file: {:?}", e);
                    return;
                }
                return;
            }
        }
    };

    let theme = if let Some(path) = args.theme_path {
        let slice = fs::read(path).expect("theme file read failed, manual spec");
        let config: serde_json::Result<HashMap<String, String>> = serde_json::from_slice(&slice);
        config.expect("serde_json::from_slice failed")
    } else {
        let mut path = configtool_dir; // no need to clone again, last usage of path
        path.push("themes");
        path.push(&config.theme_name);
        let slice = fs::read(path).expect("theme file read failed, auto spec");
        let config: serde_json::Result<HashMap<String, String>> = serde_json::from_slice(&slice);
        config.expect("serde_json::from_slice failed")
    };

    for file in config.files {
        let mut contents = fs::read_to_string(&file).expect("File should exist");
        for (key, value) in &theme {
            contents = contents.replace(key, value);
        }
        fs::write(&file, &contents).expect("File write should work");
    }
}
