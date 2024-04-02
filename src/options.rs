use crate::{error_handler, event_handler, Data};

use crate::commands::{
    other::{cleanup, help},
    owner::{ping, register, servers, uptime},
    translation::{autotranslate, translate, usage},
};

use anyhow::{Error, Result};
use std::sync::Arc;
use std::vec;
use tokio::time::Duration;
use tracing::error;

use poise::{EditTracker, FrameworkOptions, PrefixFrameworkOptions};

const EDIT_DURATION: Duration = Duration::from_secs(5 * 60); // 5 minutes

pub fn options() -> Result<FrameworkOptions<Data, Error>> {
    let options = FrameworkOptions {
        commands: vec![
            // Owner
            ping(),
            register(),
            servers(),
            uptime(),
            // Translation
            autotranslate(),
            translate(),
            usage(),
            // Other
            cleanup(),
            help(),
        ],
        prefix_options: PrefixFrameworkOptions {
            edit_tracker: Some(Arc::new(EditTracker::for_timespan(EDIT_DURATION))),
            ..Default::default()
        },
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
        on_error: |error| {
            Box::pin(async {
                error_handler(error)
                    .await
                    .unwrap_or_else(|error| error!("{:#?}", error));
            })
        },
        ..Default::default()
    };

    Ok(options)
}
