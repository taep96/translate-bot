use crate::{Context, TranslateHook, Translation};

use anyhow::Result;

use poise::command;
use poise::serenity_prelude as serenity;
use serenity::{Mention, Message};

use num_format::{Locale, ToFormattedString};

/// Toggle automatic translation in the current channel
///
/// When enabled, the bot will automatically translate messages in the channel.
#[command(
    prefix_command,
    slash_command,
    category = "Translation",
    required_bot_permissions = "MANAGE_WEBHOOKS",
    track_edits
)]
pub async fn autotranslate(ctx: Context<'_>) -> Result<()> {
    let channel_id = ctx.channel_id();
    let mention = Mention::from(channel_id);

    let channels = &ctx.data().autotranslate_channels;
    let response = |status| format!("Translation {} in {}", status, mention);

    if channels.contains_key(&channel_id) {
        channels.remove(&channel_id);
        ctx.reply(response("disabled")).await?;
    } else {
        let webhook = TranslateHook::new(ctx.serenity_context(), &channel_id).await?;
        ctx.data().enable_autotranslate(channel_id, webhook);
        ctx.reply(response("enabled")).await?;
    }

    Ok(())
}

/// Translate a message
///
/// Translates the provided message.
#[command(context_menu_command = "Translate Message", category = "Translation")]
pub async fn translate(ctx: Context<'_>, message: Message) -> Result<()> {
    ctx.reply(format!("{}\u{200b}", ctx.data().loading_emoji))
        .await?
        .delete(ctx)
        .await?;

    let translation = Translation::new(&ctx.data().deepl, &message)?;
    TranslateHook::new(ctx.serenity_context(), &message.channel_id)
        .await?
        .translate_reply(&ctx, &ctx.framework(), &message, &translation)
        .await?;

    Ok(())
}

/// Show usage
///
/// Shows the current usage of the DeepL API limit.
#[command(
    prefix_command,
    slash_command,
    category = "Translation",
    track_edits,
    ephemeral
)]
pub async fn usage(ctx: Context<'_>) -> Result<()> {
    let usage = ctx.data().deepl.get_usage().await?;
    let percentage_used = (usage.character_count as f32 / usage.character_limit as f32) * 100_f32;

    ctx.reply(format!(
        "Used `{}/{}` ({:.2}%) characters",
        usage.character_count.to_formatted_string(&Locale::en),
        usage.character_limit.to_formatted_string(&Locale::en),
        percentage_used
    ))
    .await?;

    Ok(())
}
