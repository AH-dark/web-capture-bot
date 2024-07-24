use headless_chrome::Browser;
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

    let chrome = Browser::default()?;

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .branch(dptree::case![Command::Start].endpoint(start_command_handler))
                .branch(dptree::case![Command::Help].endpoint(help_command_handler))
                .branch(dptree::case![Command::Capture].endpoint(capture_command_handler)),
        )
        .branch(Update::filter_message().filter(|message: Message| message.chat.is_private()).endpoint(handlers::private_message_handler));

    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .distribution_function(|_| None::<std::convert::Infallible>)
        .dependencies(dptree::deps![chrome])
        .build();

    if config.webhook_url.is_some() {
        let update_lisener = {
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

        dispatcher.dispatch_with_listener(update_lisener, LoggingErrorHandler::new()).await;
    } else {
        dispatcher.dispatch().await;
    }

    Ok(())
}
