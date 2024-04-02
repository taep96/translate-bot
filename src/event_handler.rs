use crate::{Data, TranslateHook, Translation};

use anyhow::{anyhow, Error, Result};

use poise::{serenity_prelude as serenity, FrameworkContext};
use serenity::FullEvent::{Message, MessageDelete, MessageUpdate};
use serenity::{Context, FullEvent};

pub async fn event_handler(
    ctx: &Context,
    event: &FullEvent,
    framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<()> {
    let deepl = &data.deepl;
    let tracker = &data.autotranslate_edit_tracker;
    let channels = &data.autotranslate_channels;

    let is_enabled = |channel_id| channels.contains_key(channel_id);
    let get_webhook = |channel_id| -> Result<TranslateHook> {
        let webhook = channels
            .get(channel_id)
            .ok_or_else(|| anyhow!("Webhook not found"))?
            .webhook
            .clone();

        Ok(webhook)
    };

    match event {
        Message {
            new_message: message,
        } => {
            let author = &message.author;
            let content = &message.content;
            let channel_id = &message.channel_id;

            if author.bot || content.is_empty() || !is_enabled(channel_id) {
                return Ok(());
            }

            let translation = Translation::new(deepl, message)?;
            let response = get_webhook(channel_id)?
                .translate_reply(ctx, &framework, message, &translation)
                .await?;

            data.track_edits(message.clone(), response);
        }

        MessageUpdate { event: message, .. } => {
            let Some(author) = &message.author else { return Ok(()) };
            let Some(content) = &message.content else { return Ok(()) };

            if author.bot || content.is_empty() {
                return Ok(());
            }

            let mut tracked = {
                let Some(tracked) = tracker.get_mut(&message.id) else { return Ok(()) };
                tracked.clone()
            };
            message.apply_to_message(&mut tracked.message);
            let translation = Translation::new(deepl, &tracked.message)?;

            let response = get_webhook(&message.channel_id)?
                .edit_response(ctx, &tracked.response, &translation)
                .await?;

            data.track_edits(tracked.message.clone(), response);
        }

        MessageDelete {
            deleted_message_id: message_id,
            ..
        } => {
            let Some((_, tracked)) = tracker.remove(message_id) else { return Ok(()) };
            get_webhook(&tracked.message.channel_id)?
                .delete(ctx, &tracked.response)
                .await?;
        }
        _ => (),
    }

    Ok(())
}
