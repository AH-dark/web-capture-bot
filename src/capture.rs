use anyhow::Context;
use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::protocol::cdp::Target::CreateTarget;

pub fn capture_website(browser: Browser, url: &str) -> anyhow::Result<Vec<u8>> {
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
