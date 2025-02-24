use anyhow::Result;
use server::config::{Config, Writer};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<(Config, Box<dyn Writer>)> {
    let config = toml::from_str::<Config>(std::fs::read_to_string(path.as_ref())?.as_str())?;
    let writer = Box::new(ConfigWriter::new(path.as_ref().to_path_buf()));
    Ok((config, writer))
}

#[derive(Clone)]
pub struct ConfigWriter {
    path: PathBuf,
}

impl ConfigWriter {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Writer for ConfigWriter {
    fn write(&self, config: &Config) -> Result<()> {
        let config_str = toml::to_string(config)?;
        let tmp_path = self.path.with_extension("tmp");
        {
            let mut file = std::fs::File::create(&tmp_path)?;
            file.write_all(config_str.as_bytes())?;
            file.sync_all()?;
        }
        std::fs::rename(tmp_path, self.path.clone())?;
        Ok(())
    }
}
