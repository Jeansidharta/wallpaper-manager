use question::{Answer, Question};
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::fs::{create_dir_all, read_to_string, write};
use std::io::ErrorKind;
use std::path::PathBuf;
use toml;

const ENV_VAR_XDG_CONFIG_DIR: &str = "XDG_CONFIG_HOME";
const ENV_VAR_HOME: &str = "HOME";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub wallpapers_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub socket_path: Option<PathBuf>,
    pub resolution: Option<ConfigResolution>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigResolution {
    pub width: i32,
    pub height: i32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            wallpapers_dir: None,
            cache_dir: None,
            socket_path: None,
            resolution: None,
        }
    }
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

    pub fn resolve_config_path_from_env() -> Result<PathBuf, ()> {
        let config_dir: PathBuf = match env::var(ENV_VAR_XDG_CONFIG_DIR) {
            Ok(v) => PathBuf::from(v),
            Err(_) => match env::var(ENV_VAR_HOME) {
                Ok(i) => PathBuf::from(i).join(".config"),
                Err(_) => return Err(()),
            },
        };

        Ok(config_dir.join("wallpaper-manager/config.toml"))
    }
}

pub fn read_config(custom_config_path: Option<PathBuf>) -> Result<Config, String> {
    let config_path =
        custom_config_path
            .ok_or(Err(()))
            .or_else(|_: Result<PathBuf, ()>| Config::resolve_config_path_from_env())
            .or_else(|_| Err(format!("Could not resolve the config directory. Either provide it as a command line argument (through --config-dir), or set either the {ENV_VAR_XDG_CONFIG_DIR} or {ENV_VAR_HOME} environment variables")))?;

    if !config_path.is_file() {
        if let Answer::YES = Question::new(&format!("Could not find a configuration file at {}. Would you like to create a default configuration at this location?", config_path.to_string_lossy())).confirm() {
            Config::default().write(&config_path)?;
            println!("Default configuration file created successfuly")
        }
        std::process::exit(0)
    };

    Config::read_from(&config_path)
}
