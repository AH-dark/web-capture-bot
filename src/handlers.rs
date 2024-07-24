use std::sync::Arc;

use moka::future::Cache;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use crate::capture::capture_website;

#[derive(Debug, Clone, BotCommands)]
#[command(rename_rule = "snake_case")]
pub enum Command {
    #[command(description = "Show this help message.")]
    Help,
    #[command(description = "Start the bot.")]
    Start,
    #[command(description = "Capture a screenshot of a website.")]
    Capture,
}

pub async fn start_command_handler(bot: Bot, message: Message) -> anyhow::Result<()> {
    bot.send_message(message.chat.id, "Hello! I am a website screenshot bot. Send me a URL and I will send you a screenshot of the website.").await?;
    Ok(())
}

pub async fn help_command_handler(bot: Bot, message: Message) -> anyhow::Result<()> {
    bot.send_message(message.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

pub async fn capture_command_handler(bot: Bot, message: Message, cache: Arc<Cache<String, Vec<u8>>>) -> anyhow::Result<()> {
    // parse the command arguments
    let args = message.text().and_then(|text| text.split_once(' ').map(|x| x.1));
    if args.is_none() {
        bot.send_message(message.chat.id, "Please provide a URL to capture a screenshot of the website.").await?;
        return Err(anyhow::anyhow!("No URL provided"));
    }

    let url = match args.unwrap().parse::<reqwest::Url>() {
        Ok(url) => url,
        Err(e) => {
            bot.send_message(message.chat.id, format!("Invalid URL: {}", e)).await?;
            return Err(e.into());
        }
    };

    // check if the screenshot is already cached
    let res = cache.get(&url.to_string()).await;
    if let Some(screenshot) = res {
        bot.send_photo(message.chat.id, teloxide::types::InputFile::memory(screenshot).file_name("screenshot.png"))
            .reply_to_message_id(message.id).await?;
        return Ok(());
    }

    let loading_msg = bot.send_message(message.chat.id, format!("Capturing a screenshot of {}...", url))
        .disable_web_page_preview(true).await?;

    // capture the screenshot
    let screenshot = match capture_website(url.as_str()).await {
        Ok(screenshot) => {
            cache.insert(url.to_string(), screenshot.clone()).await;
            screenshot
        }
        Err(e) => {
            bot.send_message(message.chat.id, format!("Failed to capture a screenshot: {}", e)).await?;
            return Err(e);
        }
    };

    // send the screenshot
    bot.send_photo(message.chat.id, teloxide::types::InputFile::memory(screenshot).file_name("screenshot.png"))
        .reply_to_message_id(message.id).await?;
    bot.delete_message(message.chat.id, loading_msg.id).await.ok(); // ignore errors

    Ok(())
}

pub async fn private_message_handler(bot: Bot, message: Message, cache: Arc<Cache<String, Vec<u8>>>) -> anyhow::Result<()> {
    let url = match message.text().map(|text| text.parse::<reqwest::Url>()) {
        Some(Ok(url)) => url,
        Some(Err(e)) => {
            bot.send_message(message.chat.id, format!("Invalid URL: {}", e)).await?;
            return Err(e.into());
        }
        None => {
            // ignore non-text messages
            return Ok(());
        }
    };

    // check if the screenshot is already cached
    let res = cache.get(&url.to_string()).await;
    if let Some(screenshot) = res {
        bot.send_photo(message.chat.id, teloxide::types::InputFile::memory(screenshot).file_name("screenshot.png"))
            .reply_to_message_id(message.id).await?;
        return Ok(());
    }

    let loading_msg = bot.send_message(message.chat.id, format!("Capturing a screenshot of {}...", url))
        .disable_web_page_preview(true).await?;

    // capture the screenshot
    let screenshot = match capture_website(url.as_str()).await {
        Ok(screenshot) => {
            cache.insert(url.to_string(), screenshot.clone()).await;
            screenshot
        }
        Err(e) => {
            bot.send_message(message.chat.id, format!("Failed to capture a screenshot: {}", e)).await?;
            return Err(e);
        }
    };

    // send the screenshot
    bot.send_photo(message.chat.id, teloxide::types::InputFile::memory(screenshot).file_name("screenshot.png"))
        .reply_to_message_id(message.id).await?;
    bot.delete_message(message.chat.id, loading_msg.id).await.ok(); // ignore errors

    Ok(())
}
