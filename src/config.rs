use anyhow::Result;

use figment::{
    providers::{Env, Format, Toml},
    Error, Figment,
};
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub deepl_auth_key: SecretString,
    pub discord_token: SecretString,
    pub loading_guild_id: u64,
    pub loading_emoji_id: u64,
}

impl Config {
    pub fn new() -> Result<Self, Error> {
        let config = Figment::new()
            .merge(Env::raw())
            .merge(Toml::file("Config.toml"))
            .extract()?;

        Ok(config)
    }
}
