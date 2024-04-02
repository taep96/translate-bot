use crate::{Config, Data};

use anyhow::{Context as _, Error, Result};
use tracing::{debug, info};

use poise::{builtins, serenity_prelude as serenity, Framework};
use serenity::{ActivityData, Context, Ready};

use colored::Colorize;
use terminal_hyperlink::Hyperlink;

pub async fn setup(
    ctx: &Context,
    ready: &Ready,
    framework: &Framework<Data, Error>,
    config: Config,
) -> Result<Data> {
    info!(
        "Logged in as {}",
        ready
            .user
            .name
            .hyperlink(format!(
                "https://discord.com/developers/applications/{}/bot",
                ready.user.id
            ))
            .underline()
            .blue()
    );

    debug!("Setting activity");
    ctx.set_activity(Some(ActivityData::listening("/help")));

    debug!("Registering commands");
    builtins::register_globally(ctx, &framework.options().commands)
        .await
        .context("Failed to register commands")?;

    debug!("Creating Data");
    Data::new(ctx, config).await
}
