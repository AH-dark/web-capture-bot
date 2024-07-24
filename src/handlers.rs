use anyhow::Context;
use headless_chrome::Browser;
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

pub async fn capture_command_handler(bot: Bot, message: Message, chrome: Browser) -> anyhow::Result<()> {
    // parse the command arguments
    let args = message.text().and_then(|text| text.split_once(' ').map(|x| x.1));
    if args.is_none() {
        bot.send_message(message.chat.id, "Please provide a URL to capture a screenshot of the website.").await?;
        return Err(anyhow::anyhow!("No URL provided"));
    }

    let url = args.unwrap().parse::<reqwest::Url>().context("Invalid URL")?;
    let loading_msg = bot.send_message(message.chat.id, format!("Capturing a screenshot of {}...", url)).await?;

    // capture the screenshot
    let screenshot = match capture_website(&chrome, url.as_str()) {
        Ok(screenshot) => screenshot,
        Err(e) => {
            bot.send_message(message.chat.id, format!("Failed to capture a screenshot: {}", e)).await?;
            return Err(e);
        }
    };

    // send the screenshot
    bot.send_photo(message.chat.id, teloxide::types::InputFile::memory(screenshot).file_name("screenshot.png")).caption(url.to_string()).await?;
    bot.delete_message(message.chat.id, loading_msg.id).await.ok(); // ignore errors

    Ok(())
}

pub async fn private_message_handler(bot: Bot, message: Message) -> anyhow::Result<()> {
    bot.send_message(message.chat.id, "I only respond to commands in private messages.").await?;
    Ok(())
}
