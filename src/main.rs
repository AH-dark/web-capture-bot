use std::sync::Arc;

use anyhow::anyhow;
use headless_chrome::{Browser, LaunchOptions};
use headless_chrome::browser::default_executable;
use moka::future::Cache;
use teloxide::prelude::*;
use teloxide::update_listeners;

use crate::handlers::{capture_command_handler, Command, help_command_handler, start_command_handler};

mod handlers;
pub mod capture;

#[derive(serde::Deserialize)]
struct Config {
    telegram_api_url: Option<String>,
    webhook_listen_addr: Option<String>,
    webhook_url: Option<String>,
    sandbox: Option<bool>,
    headless: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match dotenv::dotenv() {
        Ok(_) => log::info!("Loaded .env file"),
        Err(e) => log::warn!("Failed to load .env file: {}", e),
    }
    pretty_env_logger::init();

    log::info!("Starting website screenshot bot...");

    let config = envy::from_env::<Config>()?;

    let bot = Bot::from_env()
        .set_api_url(reqwest::Url::parse(config.telegram_api_url.unwrap_or("https://api.telegram.org".into()).as_str())?);

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .branch(dptree::case![Command::Start].endpoint(start_command_handler))
                .branch(dptree::case![Command::Help].endpoint(help_command_handler))
                .branch(dptree::case![Command::Capture].endpoint(capture_command_handler)),
        )
        .branch(Update::filter_message().filter(|message: Message| message.chat.is_private()).endpoint(handlers::private_message_handler));

    // Create a cache that can store up to 10,000 entries.
    let cache: Arc<Cache<String, Vec<u8>>> = Arc::new(
        Cache::builder()
            .max_capacity(10_000)
            .time_to_live(std::time::Duration::from_secs(60))
            .build()
    );

    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![cache])
        .build();

    if config.webhook_url.is_some() {
        let update_listener = {
            let webhook_listen_addr = config.webhook_listen_addr.unwrap_or("0.0.0.0:8080".into()).parse()?;
            log::debug!("webhook_listen_addr: {}", webhook_listen_addr);

            let webhook_url = config.webhook_url.unwrap().parse()?;
            log::debug!("webhook_url: {}", webhook_url);

            update_listeners::webhooks::axum(
                bot,
                update_listeners::webhooks::Options::new(webhook_listen_addr, webhook_url),
            )
                .await?
        };

        dispatcher.dispatch_with_listener(update_listener, LoggingErrorHandler::new()).await;
    } else {
        dispatcher.dispatch().await;
    }

    Ok(())
}
