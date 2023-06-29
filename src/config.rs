use question::{Answer, Question};
use serde_derive::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_to_string, write};
use std::io::ErrorKind;
use std::path::PathBuf;
use toml;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub wallpapers_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub socket_path: Option<PathBuf>,
    pub resolution: Option<ConfigResolution>,
    pub offset: Option<ConfigOffset>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigResolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ConfigOffset {
    pub x: i32,
    pub y: i32,
}

impl Default for ConfigResolution {
    fn default() -> Self {
        ConfigResolution {
            width: 1920,
            height: 1080,
        }
    }
}

impl Config {
    pub fn write(&self, path: &PathBuf) -> Result<(), String> {
        let parent = path
            .parent()
            .expect("Could not find configuration file parent. This is a bug");

        if !parent.is_dir() {
            create_dir_all(parent).or_else(|e| match e.kind() {
                ErrorKind::PermissionDenied => Err(format!(
                    "Permission denied when writing configuration directory at {}",
                    parent.to_string_lossy()
                )),
                _ => Err(format!(
                    "Could not create configuration file directory at {}.",
                    parent.to_string_lossy()
                )),
            })?;
        }

        write(
            path,
            toml::to_string(&self).expect("Failed to serialize configuration. This is a bug"),
        )
        .or_else(|e| match e.kind() {
            ErrorKind::PermissionDenied => Err(format!(
                "Permission denied when writing configuration at {}",
                path.to_string_lossy()
            )),
            _ => Err(format!(
                "Could not write configuration file at {}",
                path.to_string_lossy()
            )),
        })
    }

    pub fn read_from(path: &PathBuf) -> Result<Config, String> {
        if !path.is_file() {
            return Err(format!(
                "No configuration file found at {}",
                path.to_string_lossy()
            ));
        }

        let config_contents = read_to_string(path).or_else(|e| match e.kind() {
            ErrorKind::PermissionDenied => Err(format!(
                "Could not open configuration file at {}. Permission Denied",
                path.to_string_lossy()
            )),
            ErrorKind::NotFound => Err(format!(
                "Could not find a configuration file at {}",
                path.to_string_lossy()
            )),
            _ => Err(format!("")),
        })?;

        toml::from_str(&config_contents)
            .or_else(|e| Err(format!("could not parse configuration file. Error: {e}")))
    }
}

pub fn read_config(cli_config_path: Option<PathBuf>) -> Result<Config, String> {
    let config_path =
        cli_config_path
            .ok_or(dirs::config_dir())
            .or_else(|_| Err(format!("Could not resolve the config directory. Either provide it as a command line argument (through --config-dir), or set either the XDG_CONFIG_HOME or HOME environment variables")))?.join("wallpaper-manager/config.toml");

    if !config_path.is_file() {
        if let Answer::YES = Question::new(&format!("Could not find a configuration file at {}. Would you like to create a default configuration at this location?", config_path.to_string_lossy())).confirm() {
            Config::default().write(&config_path)?;
            println!("Default configuration file created successfuly")
        }
        std::process::exit(0)
    };

    Config::read_from(&config_path)
}
