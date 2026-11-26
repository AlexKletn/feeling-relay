use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::Config;

pub const DEFAULT_PROFILE_PATH: &str = "/etc/feeling-relay/feeling-relay.toml";
const TEMPLATE: &str = include_str!("template.toml");

pub fn load(path: &Path) -> Result<Option<Config>, Box<dyn Error>> {
    let document = match fs::read_to_string(path) {
        Ok(document) => document,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if create_profile(path)? {
                eprintln!("feeling-relay: created {}", path.display());
            }
            fs::read_to_string(path)?
        }
        Err(error) => return Err(error.into()),
    };
    let mut document = document.parse::<toml::Table>()?;
    match document.remove("enabled") {
        Some(toml::Value::Boolean(false)) => {
            eprintln!("feeling-relay: profile disabled at {}", path.display());
            Ok(None)
        }
        Some(toml::Value::Boolean(true)) | None => {
            let config: Config = document.try_into()?;
            config.validate()?;
            Ok(Some(config))
        }
        Some(_) => {
            Err(io::Error::new(io::ErrorKind::InvalidData, "enabled must be a boolean").into())
        }
    }
}

fn create_profile(path: &Path) -> io::Result<bool> {
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let mut template = tempfile::NamedTempFile::new_in(parent)?;
    template.write_all(TEMPLATE.as_bytes())?;
    template.as_file().sync_all()?;
    match template.persist_noclobber(path) {
        Ok(_) => Ok(true),
        Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) => Err(error.error),
    }
}

#[cfg(test)]
#[path = "file.tests.rs"]
mod tests;
