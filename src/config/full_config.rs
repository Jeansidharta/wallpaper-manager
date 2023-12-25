use question::{Answer, Question};
use serde_derive::{Deserialize, Serialize};
use std::{fs::create_dir_all, path::PathBuf};
use thiserror::Error;

use crate::Args;

use super::config_file::{ConfigFile, ConfigFileWriteError};

#[derive(Serialize, Deserialize)]
pub struct ConfigResolution {
    pub width: i32,
    pub height: i32,
}

impl Default for ConfigResolution {
    fn default() -> Self {
        ConfigResolution {
            width: 1920,
            height: 1080,
        }
    }
}

const ENV_VAR_XDG_CONFIG_DIR: &str = "XDG_CONFIG_HOME";
const ENV_VAR_HOME: &str = "HOME";

#[derive(Serialize, Deserialize, Default)]
pub struct FullConfig {
    pub socket_path: Option<PathBuf>,

    pub cache_dir: PathBuf,
    pub resolution: ConfigResolution,
    pub wallpapers_dir: PathBuf,
    pub thumbnails_cache_dir: PathBuf,
    pub wallpapers_rescaled_dir: PathBuf,
}

#[derive(Debug, Error)]
pub enum ConfigReadError {
    #[error("Could not resolve the config directory. Either provide it as a command line argument (through --config-dir), or set either the {ENV_VAR_XDG_CONFIG_DIR} or {ENV_VAR_HOME} environment variables")]
    NoConfigPath,
    #[error("Could not resolve the cache directory. Provide it in the configuration file or through --cache-dir")]
    CacheDirNotFound,
    #[error("Could not resolve the wallpapers directory. Provide it in the configuration file or through --wallpapers-dir")]
    WallpaperDirNotProvided,
    #[error("Failed to create thumbnails cache directory: {0}")]
    FailedCreateThumbnailDir(std::io::Error),
    #[error("Failed to create wallpapers rescaled cache directory: {0}")]
    FailedCreateWallpapersRescaleDir(std::io::Error),
    #[error("Wallpapers directory does not exist")]
    WallpaperDirNotFound,

    #[error("{0}")]
    ConfigWriteError(ConfigFileWriteError),
}

pub fn read(args: &Args) -> Result<FullConfig, ConfigReadError> {
    let config_file_path = args
        .config_dir
        .clone()
        .or_else(ConfigFile::resolve_config_path_from_env)
        .or_else(|| dirs::config_dir().map(|c| c.join("/")))
        .ok_or(ConfigReadError::NoConfigPath)?;

    if !config_file_path.is_file() {
        if let Answer::YES = Question::new(&format!("Could not find a configuration file at {}. Would you like to create a default configuration at this location?", config_file_path.to_string_lossy())).confirm() {
            ConfigFile::default().write(&config_file_path).map_err(ConfigReadError::ConfigWriteError)?;
            println!("Default configuration file created successfuly")
        }
        std::process::exit(0)
    };

    let config_file = ConfigFile::read_from(&config_file_path).unwrap();
    let cache_dir = args
        .cache_dir
        .clone()
        .or(config_file.cache_dir)
        .or_else(dirs::cache_dir)
        .ok_or(ConfigReadError::CacheDirNotFound)?;

    let wallpapers_dir = args
        .wallpapers_dir
        .clone()
        .or(config_file.wallpapers_dir)
        .ok_or(ConfigReadError::WallpaperDirNotProvided)?;

    let thumbnails_cache_dir = cache_dir.join("wallpapers-thumbnail");
    let wallpapers_rescaled_dir = cache_dir.join("wallpapers-rescaled");

    if !thumbnails_cache_dir.is_dir() {
        create_dir_all(&thumbnails_cache_dir).map_err(ConfigReadError::FailedCreateThumbnailDir)?
    }
    if !wallpapers_rescaled_dir.is_dir() {
        create_dir_all(&wallpapers_rescaled_dir)
            .map_err(ConfigReadError::FailedCreateWallpapersRescaleDir)?
    }

    if !wallpapers_dir.is_dir() {
        return Err(ConfigReadError::WallpaperDirNotFound);
    }

    Ok(FullConfig {
        cache_dir,
        socket_path: config_file.socket_path,
        resolution: config_file.resolution.unwrap_or_default(),
        wallpapers_dir,
        thumbnails_cache_dir,
        wallpapers_rescaled_dir,
    })
}
