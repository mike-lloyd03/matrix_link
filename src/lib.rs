use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub server_url: String,
    pub room_name: String,
}

/// Loads configuration information from a yaml file. Paths are provided as a `Vec<&str>` of
/// locations where the configuration file can be found. The first match returned will be used.
///
/// # Examples
///
/// ```
/// use matrix_link::load_config;
///
/// assert_eq!(load_config(paths), );
/// ```
///
/// # Errors
///
/// This function will return an error if no configuration file can be found or if the located
/// configuration file cannot be desirialized.
pub fn load_config(paths: Vec<&str>) -> Result<Config> {
    use std::fs::File;
    use std::path::Path;

    let f: File;

    for path in paths {
        if Path::new(path).exists() {
            f = File::open(path)?;
            return Ok(serde_yaml::from_reader(f)?);
        }
    }
    bail!("No configuration file was found")
}
