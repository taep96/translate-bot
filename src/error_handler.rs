use crate::{Data, TranslateHook, Translation};

use anyhow::{Error, Result};

use poise::serenity_prelude as serenity;
use poise::CreateReply;
use serenity::{CreateAllowedMentions, Mentionable};

type FrameworkContext<'a> = poise::FrameworkContext<'a, Data, Error>;
type FrameworkError<'a> = poise::FrameworkError<'a, Data, Error>;

pub async fn error_handler(error: FrameworkError<'_>) -> Result<()> {
    match error {
        FrameworkError::NotAnOwner { ctx, .. } => {
            ctx.send(
                ctx.reply_builder(CreateReply::default())
                    .allowed_mentions(CreateAllowedMentions::new())
                    .content(mini_help(ctx.framework()))
                    .reply(true),
            )
            .await?;
        }
        FrameworkError::UnknownCommand {
            ctx,
            framework,
            msg: message,
            ..
        } => {
            let mention = ctx.cache.current_user().mention().to_string();

            if message.referenced_message.is_none() && message.content == mention {
                message.reply(ctx, mini_help(framework)).await?;
                return Ok(());
            }

            let message = match &message.referenced_message {
                Some(referenced) => referenced,
                None => message,
            };

            let translation = Translation::new(&framework.user_data.deepl, message)?;
            TranslateHook::new(ctx, &message.channel_id)
                .await?
                .translate_reply(ctx, &framework, message, &translation)
                .await?;
        }
        FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            ctx.reply(format!(
                "I'm missing the {} permissions to do that. Please fix this and try again.",
                missing_permissions
                    .iter_names()
                    .map(|permission| format!("`{}`", permission.0))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .await?;
        }
        other => poise::builtins::on_error(other).await?,
    }

    Ok(())
}

fn mini_help(framework: FrameworkContext<'_>) -> String {
    format!(
        "Available commands: {}, use `/help` for more",
        framework
            .options
            .commands
            .iter()
            .filter(|command| !command.hide_in_help)
            .filter(|command| command.name != "help")
            .map(|command| format!("`{}`", command.name))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
