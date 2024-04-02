use crate::{Context, TranslateHook};

use anyhow::Result;

use poise::{builtins, command, serenity_prelude as serenity};
use serenity::GetMessages;

/// Show a help message
///
/// Provides help for all or a specific command.
#[command(prefix_command, slash_command, category = "Other", track_edits)]
pub async fn help(
    ctx: Context<'_>,
    #[autocomplete = "builtins::autocomplete_command"] command: Option<String>,
) -> Result<()> {
    builtins::help(
        ctx,
        command.as_deref(),
        builtins::HelpConfiguration {
            show_context_menu_commands: true,
            extra_text_at_bottom: "You can also ping me to translate messages or use commands",
            ..Default::default()
        },
    )
    .await?;

    Ok(())
}

/// Delete bot's messages
///
/// Deletes the provided amount of messages sent by the bot in the last 24 hours. (default `1`)
#[command(
    prefix_command,
    slash_command,
    category = "Other",
    required_bot_permissions = "MANAGE_MESSAGES",
    track_edits,
    ephemeral
)]
pub async fn cleanup(
    ctx: Context<'_>,
    #[min = 0]
    #[max = 100]
    amount: Option<usize>,
) -> Result<()> {
    let amount = amount.unwrap_or(1);
    let webhook_user_id: u64 = TranslateHook::new(ctx.serenity_context(), &ctx.channel_id())
        .await?
        .webhook
        .id
        .into();

    let messages = ctx
        .channel_id()
        .messages(&ctx, GetMessages::new().limit(100))
        .await?
        .into_iter()
        .filter(|message| {
            let is_recent = (*ctx.created_at() - *message.timestamp).num_hours() < 24;
            let is_from_bot = message.author.id == ctx.framework().bot_id;
            let is_from_webhook = message.author.id == webhook_user_id;

            is_recent && (is_from_bot || is_from_webhook)
        })
        .take(amount);

    ctx.channel_id().delete_messages(ctx, messages).await?;
    ctx.reply(format!("Deleted {} messages", amount)).await?;

    Ok(())
}
