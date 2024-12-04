use std::{
    env,
    fs::{create_dir_all, read_to_string, write},
    io::ErrorKind,
    path::PathBuf,
};

use serde_derive::{Deserialize, Serialize};

use super::full_config::ConfigResolution;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ConfigFile {
    pub wallpapers_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub socket_path: Option<PathBuf>,
    pub resolution: Option<ConfigResolution>,
}

const ENV_VAR_XDG_CONFIG_DIR: &str = "XDG_CONFIG_HOME";
const ENV_VAR_HOME: &str = "HOME";

#[derive(Debug, thiserror::Error)]
pub enum ConfigFileReadError {
    #[error("No configuration file found at {0}")]
    ConfigFileNotFound(PathBuf),
    #[error("Could not open configuration file at {0}. Permission Denied")]
    ConfigFilePermissionDenied(PathBuf),
    #[error("Failed to parse configuration file. Error: {0}")]
    ParsingError(toml::de::Error),
    #[error("{0}")]
    IoError(std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigFileWriteError {
    #[error("Permission denied when writing configuration directory at {0}")]
    DirPermissionDenied(std::io::Error),
    #[error("Could not create configuration file directory at {0}: {1}")]
    IoCreateDirError(PathBuf, std::io::Error),
    #[error("Could not write configuration file at {0}: {1}")]
    IoWriteFileError(PathBuf, std::io::Error),

    #[error("Permission denied when writing configuration at {0}: {1}")]
    FileWriteFileError(PathBuf, std::io::Error),
}
impl ConfigFile {
    pub fn write(&self, path: &PathBuf) -> Result<(), ConfigFileWriteError> {
        let parent = path.parent().unwrap();

        if !parent.is_dir() {
            create_dir_all(parent).map_err(|e| match e.kind() {
                ErrorKind::PermissionDenied => ConfigFileWriteError::DirPermissionDenied(e),
                _ => ConfigFileWriteError::IoCreateDirError(parent.to_path_buf(), e),
            })?;
        }

        write(path, toml::to_string(&self).unwrap()).map_err(|e| match e.kind() {
            ErrorKind::PermissionDenied => {
                ConfigFileWriteError::FileWriteFileError(path.to_path_buf(), e)
            }
            _ => ConfigFileWriteError::IoWriteFileError(path.to_path_buf(), e),
        })
    }
    pub fn read_from(path: &PathBuf) -> Result<ConfigFile, ConfigFileReadError> {
        if !path.is_file() {
            return Err(ConfigFileReadError::ConfigFileNotFound(path.clone()));
        }

        let config_contents = read_to_string(path).map_err(|e| match e.kind() {
            ErrorKind::PermissionDenied => {
                ConfigFileReadError::ConfigFilePermissionDenied(path.to_path_buf())
            }
            ErrorKind::NotFound => ConfigFileReadError::ConfigFileNotFound(path.to_path_buf()),
            _ => ConfigFileReadError::IoError(e),
        })?;

        toml::from_str(&config_contents).map_err(ConfigFileReadError::ParsingError)
    }

    pub fn resolve_config_path_from_env() -> Option<PathBuf> {
        let config_dir: PathBuf = match env::var(ENV_VAR_XDG_CONFIG_DIR) {
            Ok(v) => PathBuf::from(v),
            Err(_) => match env::var(ENV_VAR_HOME) {
                Ok(i) => PathBuf::from(i).join(".config"),
                Err(_) => return None,
            },
        };

        Some(config_dir.join("wallpaper-manager/config.toml"))
    }
}
