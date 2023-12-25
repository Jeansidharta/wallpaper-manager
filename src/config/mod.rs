mod config_file;
mod full_config;

pub use full_config::read as read_config;
pub use full_config::ConfigResolution;
pub use full_config::FullConfig as Config;
