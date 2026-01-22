use crate::{
    Result,
    error::{Error, FlattenError},
};
use cfg_rs::{Configuration, FromConfig};
use exn::ResultExt;
use std::fmt::Debug;

pub trait IsConfig: FromConfig {}

impl<T> IsConfig for T where T: FromConfig {}

pub trait ConfigSource {
    fn get_config<T: IsConfig>(&self, key: impl AsRef<str>) -> Result<T>;
    fn get_config_or<T: IsConfig>(&self, key: impl AsRef<str>, default: T) -> Result<T>;
}

pub struct CfgSource {
    conf: Configuration,
}
impl Debug for CfgSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CfgSource {{ ... }}")
    }
}

#[derive(Debug)]
pub struct CfgParams<'a> {
    pub name: &'a str,
    pub dir: &'a str,
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
