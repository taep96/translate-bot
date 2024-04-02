use anyhow::{bail, Result};

use poise::serenity_prelude as serenity;
use serenity::Message;

use deepl::DeepLApi;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

pub struct TargetLanguage {
    pub lang: deepl::Lang,
    pub flag: &'static str,
}

impl TargetLanguage {
    fn from_lang(lang: Lang) -> Self {
        match lang {
            Lang::JA => Self {
                lang: deepl::Lang::EN,
                flag: "🇬🇧",
            },
            Lang::EN => Self {
                lang: deepl::Lang::JA,
                flag: "🇯🇵",
            },
        }
    }
}

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"https?://\S+").unwrap();
    static ref EVERYONE_REGEX: Regex = Regex::new(r"@(everyone|here)").unwrap();
    static ref MENTION_REGEX: Regex = Regex::new(r"<(@&|@|#)\d+>").unwrap();
    static ref EMOJI_REGEX: Regex = Regex::new(r"<:\w+:\d+>").unwrap();
    static ref TIMESTAMP_REGEX: Regex = Regex::new(r"<t:\d+:?\w>").unwrap();
    static ref CODE_BLOCK_REGEX: Regex = RegexBuilder::new(r"```.*```")
        .dot_matches_new_line(true)
        .build()
        .unwrap();
}

pub struct Translation<'a> {
    deepl: &'a DeepLApi,
    message: &'a Message,
    pub target: TargetLanguage,
}

impl<'a> Translation<'a> {
    pub fn new(deepl: &'a DeepLApi, message: &'a Message) -> Result<Self> {
        let mut without_matches = message.content.clone();

        for regex in [
            &*EVERYONE_REGEX,
            &*MENTION_REGEX,
            &*EMOJI_REGEX,
            &*TIMESTAMP_REGEX,
            &*URL_REGEX,
            &*CODE_BLOCK_REGEX,
        ] {
            without_matches = regex.replace_all(&without_matches, "").into();
        }
        without_matches = without_matches.trim().into();

        if without_matches.is_empty() | without_matches.chars().all(|c| !c.is_alphabetic()) {
            bail!("Nothing to translate");
        }

        let original = detect_language(&without_matches);
        let target = TargetLanguage::from_lang(original);

        Ok(Self {
            deepl,
            message,
            target,
        })
    }

    pub async fn translate(&self) -> Option<String> {
        self.deepl
            .translate_text(&self.message.content, self.target.lang.clone())
            .await
            .map(|translation| translation.to_string())
            .ok()
    }
}

enum Lang {
    EN,
    JA,
}

fn detect_language(text: &str) -> Lang {
    let ascii_count = text.chars().filter(|c| c.is_ascii_alphanumeric()).count();

    if ascii_count > text.len() / 2 {
        Lang::EN
    } else {
        Lang::JA
    }
}
