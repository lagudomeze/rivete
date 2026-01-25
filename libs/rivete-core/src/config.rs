use crate::{
    error::{Error, FlattenError},
    Result,
};
use cfg_rs::{Configuration, FromConfig};
use exn::ResultExt;
use std::fmt::Debug;

/// Marker trait for types that can be loaded from configuration.
///
/// This trait is automatically implemented for all types that implement [`FromConfig`].
pub trait IsConfig: FromConfig {}

impl<T> IsConfig for T where T: FromConfig {}

/// Trait for sources that provide configuration values.
pub trait ConfigSource {
    /// Retrieves a configuration value for the given key.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ConfigError`] if the key is not found or the value cannot be parsed.
    fn get_config<T: IsConfig>(&self, key: impl AsRef<str>) -> Result<T>;

    /// Retrieves a configuration value for the given key, or returns the default value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ConfigError`] if the key exists but the value cannot be parsed.
    fn get_config_or<T: IsConfig>(&self, key: impl AsRef<str>, default: T) -> Result<T>;
}

/// Configuration source backed by a [`Configuration`] instance.
pub struct CfgSource {
    conf: Configuration,
}
impl Debug for CfgSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CfgSource {{ ... }}")
    }
}

/// Parameters for initializing a configuration source.
#[derive(Debug)]
pub struct CfgParams<'a> {
    /// Application name used for configuration file naming.
    pub name: &'a str,
    /// Directory to search for configuration files.
    pub dir: &'a str,
    /// Prefix for environment variables.
    pub prefix_env: &'a str,
}

impl Default for CfgParams<'static> {
    fn default() -> Self {
        Self {
            name: "app",
            dir: ".",
            prefix_env: "APP",
        }
    }
}

impl CfgSource {
    /// Creates a new configuration source with the given parameters.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ConfigError`] if the configuration cannot be loaded.
    pub fn new(param: CfgParams) -> Result<Self> {
        let conf = Configuration::with_predefined_builder()
            .set_prefix_env(param.prefix_env)
            .set_name(param.name)
            .set_dir(param.dir)
            .init()
            .map_err(FlattenError::from)
            .or_raise(|| Error::ConfigError)?;

        Ok(Self { conf })
    }
}

impl ConfigSource for CfgSource {
    fn get_config<T: IsConfig>(&self, key: impl AsRef<str>) -> Result<T> {
        Ok(self
            .conf
            .get::<T>(key.as_ref())
            .map_err(FlattenError::from)
            .or_raise(|| Error::ConfigError)?)
    }

    fn get_config_or<T: IsConfig>(&self, key: impl AsRef<str>, default: T) -> Result<T> {
        Ok(self
            .conf
            .get_or::<T>(key.as_ref(), default)
            .map_err(FlattenError::from)
            .or_raise(|| Error::ConfigError)?)
    }
}
