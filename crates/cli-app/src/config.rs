use anyhow::Result;
use server::config::Config;
use std::path::Path;

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
    let config = toml::from_str::<Config>(std::fs::read_to_string(path)?.as_str())?;
    Ok(config)
}
