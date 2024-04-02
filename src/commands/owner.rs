use crate::Context;

use anyhow::Result;
use tokio::time::Duration;

use fancy_duration::AsFancyDuration;
use poise::{builtins, command};

#[command(
    prefix_command,
    category = "Owner",
    owners_only,
    hide_in_help,
    track_edits
)]
pub async fn register(ctx: Context<'_>) -> Result<()> {
    builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[command(
    prefix_command,
    category = "Owner",
    owners_only,
    hide_in_help,
    track_edits
)]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
    let ping_start = std::time::SystemTime::now();
    ctx.reply("Pinging...").await?;

    let elapsed = ping_start.elapsed()?.as_millis();
    ctx.reply(format!("Pong! {}ms", elapsed)).await?;

    Ok(())
}

#[command(
    prefix_command,
    category = "Owner",
    owners_only,
    hide_in_help,
    track_edits
)]
pub async fn uptime(ctx: Context<'_>) -> Result<()> {
    let duration = Duration::from_secs(ctx.data().start_time.elapsed().as_secs());
    ctx.reply(format!("{}", duration.fancy_duration())).await?;

    Ok(())
}

#[command(
    prefix_command,
    category = "Owner",
    owners_only,
    hide_in_help,
    track_edits
)]
pub async fn servers(ctx: Context<'_>) -> Result<()> {
    builtins::servers(ctx).await?;
    Ok(())
}
