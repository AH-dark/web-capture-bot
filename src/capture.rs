use anyhow::{anyhow, Context};
use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::protocol::cdp::Target::CreateTarget;

fn new_browser() -> anyhow::Result<Browser> {
    let launch_options = headless_chrome::LaunchOptions::default_builder()
        .path(Some(headless_chrome::browser::default_executable().map_err(|e| anyhow!(e))?))
        .sandbox(std::env::var("SANDBOX").unwrap_or("true".into()) == "true")
        .headless(std::env::var("HEADLESS").unwrap_or("true".into()) == "true")
        .window_size(Some((1920, 1080)))
        .enable_logging(true)
        .idle_browser_timeout(std::time::Duration::from_secs(60 * 30))
        .build()?;

    Browser::new(launch_options)
}

pub fn capture_website(url: &str) -> anyhow::Result<Vec<u8>> {
    let browser = new_browser().context("Failed to create a new browser")?;

    let tab = browser.new_tab_with_options(CreateTarget {
        url: "about:blank".into(),
        width: Some(1920),
        height: Some(1080),
        browser_context_id: None,
        enable_begin_frame_control: None,
        new_window: None,
        background: None,
    }).context("Failed to create a new tab")?;
    tab.set_default_timeout(std::time::Duration::from_secs(10));

    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;

    let screenshot = tab.capture_screenshot(
        CaptureScreenshotFormatOption::Png,
        Some(100),
        None,
        true,
    )?;

    Ok(screenshot)
}
