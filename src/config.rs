extern crate toml;

use std::net::IpAddr;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std;

use serde::{de, Deserialize, Deserializer};

use trust_dns::rr::Name;

fn deserialize_name<'de, D>(deserializer: D) -> Result<Name, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
}

quick_error! {
    #[derive(Debug)]
    pub enum ConfigError {
        Io(err: std::io::Error) {
            cause(err)
            description(err.description())
            display("I/O error: {}", err)
            from()
        }
        Toml(err: toml::de::Error) {
            cause(err)
            description(err.description())
            display("config syntax error: {}", err)
            from()
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub inwx: Inwx,
    pub api: Api,
    pub hosts: Vec<Host>
}

#[derive(Deserialize, Debug)]
pub struct Inwx {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct Api {
    pub bind: IpAddr,
    pub port: u16,
}

#[derive(Deserialize, Debug)]
pub struct Host {
    pub domain_id: u32,
    #[serde(deserialize_with = "deserialize_name")]
    pub domain_name: Name,
    pub token: String,
}

pub fn get_config(path: String) -> Result<Config, ConfigError> {
    let mut config_toml = String::new();
    let mut file = File::open(&path)?;
    file.read_to_string(&mut config_toml)?;
    let config: Config = toml::from_str(&config_toml[..])?;
    Ok(config)
}
