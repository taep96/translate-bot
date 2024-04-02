use crate::{Config, TranslateHook};

use anyhow::Result;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};

use poise::serenity_prelude as serenity;
use serenity::{ChannelId, Context, Emoji, EmojiId, GuildId, Message, MessageId};

use dashmap::DashMap;
use deepl::DeepLApi;
use secrecy::ExposeSecret;

const EDIT_DURATION: Duration = Duration::from_secs(5 * 60); // 10 minutes

pub struct Data {
    pub deepl: DeepLApi,
    pub loading_emoji: Emoji,
    pub autotranslate_channels: DashMap<ChannelId, Channel>,
    pub autotranslate_edit_tracker: Arc<DashMap<MessageId, TrackedMessage>>,
    pub config: Config,
    pub start_time: Instant,
}

impl Data {
    pub async fn new(ctx: &Context, config: Config) -> Result<Self> {
        let autotranslate_edit_tracker = Arc::new(DashMap::new());
        let tracker = autotranslate_edit_tracker.clone();

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(10)).await;
                tracker.retain(|_, v: &mut TrackedMessage| !v.is_expired());
                tracker.shrink_to_fit();
            }
        });

        let loading_emoji = ctx
            .http
            .get_emoji(
                GuildId::new(config.loading_guild_id),
                EmojiId::new(config.loading_emoji_id),
            )
            .await?;

        Ok(Self {
            deepl: DeepLApi::with(config.deepl_auth_key.expose_secret()).new(),
            loading_emoji,
            autotranslate_channels: DashMap::new(),
            autotranslate_edit_tracker,
            config,
            start_time: Instant::now(),
        })
    }

    pub fn enable_autotranslate(&self, channel_id: ChannelId, webhook: TranslateHook) {
        self.autotranslate_channels
            .insert(channel_id, Channel::new(webhook));
    }

    pub fn track_edits(&self, message: Message, response: Message) {
        self.autotranslate_edit_tracker
            .insert(message.id, TrackedMessage::new(message, response));
    }
}

pub struct Channel {
    pub webhook: TranslateHook,
}

#[allow(clippy::new_without_default)]
impl Channel {
    pub fn new(webhook: TranslateHook) -> Self {
        Self { webhook }
    }
}

#[derive(Clone)]
pub struct TrackedMessage {
    pub message: Message,
    pub response: Message,
    start_time: Instant,
}

impl TrackedMessage {
    pub fn new(message: Message, response: Message) -> Self {
        Self {
            message,
            response,
            start_time: Instant::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.start_time.elapsed() > EDIT_DURATION
    }
}
