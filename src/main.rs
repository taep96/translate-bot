mod commands;
mod config;
mod data;
mod error_handler;
mod event_handler;
mod options;
mod setup;
mod translation;
mod webhook;

use error_handler::error_handler;
use event_handler::event_handler;
use options::options;
use setup::setup;

use config::Config;
use data::Data;
use translation::Translation;
use webhook::TranslateHook;

use anyhow::{Error, Result};
use tracing::{debug, error};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use poise::serenity_prelude as serenity;
use poise::Framework;
use secrecy::ExposeSecret;
use serenity::{ClientBuilder, GatewayIntents};

type Context<'a> = poise::Context<'a, Data, Error>;

async fn run() -> Result<()> {
    debug!("Loading secrets");
    let config = Config::new()?;
    let token = config.discord_token.clone();

    debug!("Building framework");
    let framework = Framework::builder()
        .options(options()?)
        .setup(|ctx, ready, framework| Box::pin(setup(ctx, ready, framework, config)))
        .build();

    use GatewayIntents as Intents;
    let intents = Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

    debug!("Starting up");
    ClientBuilder::new(token.expose_secret(), intents)
        .framework(framework)
        .await?
        .start()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::try_new("info,serenity=warn").expect("Invalid filter"));

    let subscriber =
        tracing_subscriber::registry().with(tracing_subscriber::fmt::layer().with_filter(filter));

    #[cfg(debug_assertions)]
    let subscriber = subscriber.with(console_subscriber::spawn());

    subscriber.init();

    if let Err(error) = run().await {
        error!("{:#?}", error);
        std::process::exit(1);
    }

    Ok(())
}
