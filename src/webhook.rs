use crate::{Data, Translation};

use anyhow::{anyhow, Context as _, Error, Result};
use tracing::error;

use poise::serenity_prelude as serenity;
use poise::FrameworkContext;
use serenity::{
    CacheHttp, Channel, ChannelId, ChannelType, CreateAllowedMentions, CreateAttachment,
    CreateWebhook, EditWebhookMessage, ExecuteWebhook, Message, MessageFlags, User, Webhook,
};

#[derive(Clone)]
pub struct TranslateHook {
    pub webhook: Webhook,
}

impl TranslateHook {
    pub async fn new(ctx: &serenity::Context, channel_id: &ChannelId) -> Result<Self> {
        let current_user = ctx
            .cache()
            .context("Couldn't get cache")?
            .current_user()
            .clone();
        let name = format!("auto-translate-{}", current_user.id);

        let webhook_channel = get_parent(channel_id.to_channel(ctx).await?).unwrap_or(*channel_id);
        let webhook = match webhook_channel
            .webhooks(ctx)
            .await?
            .into_iter()
            .find(|webhook| webhook.name == Some(name.clone()))
        {
            Some(webhook) => webhook,
            None => {
                let avatar = CreateAttachment::url(ctx, &avatar_url(&current_user.into())).await?;

                webhook_channel
                    .create_webhook(ctx, CreateWebhook::new(name).avatar(&avatar))
                    .await?
            }
        };

        Ok(Self { webhook })
    }

    pub async fn translate_reply(
        &self,
        ctx: &impl CacheHttp,
        framework: &FrameworkContext<'_, Data, Error>,
        message: &Message,
        translation: &Translation<'_>,
    ) -> Result<Message> {
        let avatar_url = avatar_url(&message.author);

        let username = message.author_nick(ctx).await.unwrap_or_else(|| {
            let author = message.author.clone();
            author.global_name.unwrap_or(author.name)
        });
        let username = format!("{} {}", username, translation.target.flag);

        let builder = ExecuteWebhook::new()
            .avatar_url(avatar_url)
            .username(username)
            .content(format!("{}\u{200b}", framework.user_data.loading_emoji))
            .flags(MessageFlags::SUPPRESS_EMBEDS)
            .allowed_mentions(CreateAllowedMentions::new());
        let builder = match get_parent(message.channel(ctx).await?) {
            Some(_) => builder.in_thread(message.channel_id),
            None => builder,
        };

        let response = self
            .webhook
            .execute(ctx, true, builder)
            .await?
            .ok_or_else(|| anyhow!("Webhook response is empty"))?;

        let edited = self.edit_response(ctx, &response, translation).await?;

        Ok(edited)
    }

    pub async fn edit_response(
        &self,
        ctx: &impl CacheHttp,
        message: &Message,
        translation: &Translation<'_>,
    ) -> Result<Message> {
        let builder = edit_builder_parent(
            ctx,
            message,
            &translation
                .translate()
                .await
                .unwrap_or_else(|| "Translation failed.".to_string()),
        )
        .await?;

        let edit = |builder| self.webhook.edit_message(ctx, message.id, builder);
        let response = match edit(builder).await {
            Ok(edited) => edited,
            Err(err) => {
                let error_message = if err.to_string() == "Message too large." {
                    "Translation too long."
                } else {
                    error!("{}", err);
                    "Translation failed."
                };

                let builder = edit_builder_parent(ctx, message, error_message).await?;
                edit(builder).await?
            }
        };

        Ok(response)
    }

    pub async fn delete(&self, ctx: &serenity::Context, message: &Message) -> Result<()> {
        let thread_id = get_parent(message.channel(ctx).await?).map(|_| message.channel_id);
        self.webhook
            .delete_message(ctx, thread_id, message.id)
            .await?;
        Ok(())
    }
}

fn avatar_url(user: &User) -> String {
    user.avatar_url()
        .unwrap_or_else(|| user.default_avatar_url())
}

fn get_parent(channel: Channel) -> Option<ChannelId> {
    let guild_channel = channel.guild()?;

    if matches!(
        guild_channel.kind,
        ChannelType::PublicThread | ChannelType::PrivateThread
    ) {
        return guild_channel.parent_id;
    }
    None
}

async fn edit_builder_parent(
    ctx: &impl CacheHttp,
    message: &Message,
    content: &str,
) -> Result<EditWebhookMessage> {
    let builder = EditWebhookMessage::new()
        .content(content)
        .allowed_mentions(CreateAllowedMentions::new());
    match get_parent(message.channel(ctx).await?) {
        Some(_) => Ok(builder.in_thread(message.channel_id)),
        None => Ok(builder),
    }
}
