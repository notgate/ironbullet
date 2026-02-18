use futures::StreamExt;
use uuid::Uuid;

use super::*;

impl ExecutionContext {
    pub(super) async fn execute_browser_open(&mut self, settings: &BrowserOpenSettings) -> crate::error::Result<()> {
        use chromiumoxide::browser::{Browser, BrowserConfig};

        let mut builder = BrowserConfig::builder();
        if !settings.headless {
            builder = builder.with_head();
        }
        if !settings.proxy.is_empty() {
            let proxy = self.variables.interpolate(&settings.proxy);
            builder = builder.arg(format!("--proxy-server={}", proxy));
        }
        if !settings.extra_args.is_empty() {
            for arg in settings.extra_args.split_whitespace() {
                builder = builder.arg(arg);
            }
        }

        let config = builder.build()
            .map_err(|e| crate::error::AppError::Pipeline(format!("Browser config error: {}", e)))?;

        let (browser, mut handler) = Browser::launch(config).await
            .map_err(|e| crate::error::AppError::Pipeline(format!("Browser launch failed: {}", e)))?;

        // Spawn CDP event handler in background -- must NOT break on errors
        // or all future browser commands fail with "oneshot cancelled"
        tokio::spawn(async move {
            while handler.next().await.is_some() {}
        });

        self.browser = BrowserHandle(Some(browser));
        self.page = PageHandle(None);
        Ok(())
    }

    pub(super) async fn execute_navigate_to(&mut self, settings: &NavigateToSettings, block_id_for_nav: Uuid, block_label_for_nav: String) -> crate::error::Result<()> {
        let browser = self.browser.0.as_ref()
            .ok_or_else(|| crate::error::AppError::Pipeline("No browser open. Use BrowserOpen first.".into()))?;

        let url = self.variables.interpolate(&settings.url);
        let nav_start = std::time::Instant::now();

        // Inject custom cookies via CDP before navigation
        if !settings.custom_cookies.is_empty() {
            let cookie_str = self.variables.interpolate(&settings.custom_cookies);
            // Need a page to set cookies on - create one if needed
            let page_for_cookies = if let Some(ref existing) = self.page.0 {
                existing.clone()
            } else {
                browser.new_page("about:blank").await
                    .map_err(|e| crate::error::AppError::Pipeline(format!("New page failed: {}", e)))?
            };
            let domain = reqwest::Url::parse(&url).ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .unwrap_or_default();
            for line in cookie_str.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
                if let Some((name, value)) = line.split_once('=') {
                    let params = chromiumoxide::cdp::browser_protocol::network::SetCookieParams::builder()
                        .name(name.trim())
                        .value(value.trim())
                        .domain(domain.clone())
                        .build()
                        .unwrap();
                    let _ = page_for_cookies.execute(params).await;
                }
            }
            if self.page.0.is_none() {
                self.page = PageHandle(Some(page_for_cookies));
            }
        }

        let page = if let Some(ref existing) = self.page.0 {
            existing.goto(&url).await
                .map_err(|e| crate::error::AppError::Pipeline(format!("Navigate failed: {}", e)))?;
            existing.clone()
        } else {
            browser.new_page(&url).await
                .map_err(|e| crate::error::AppError::Pipeline(format!("New page failed: {}", e)))?
        };

        // Wait for page to load
        let _ = page.wait_for_navigation().await;
        let nav_elapsed = nav_start.elapsed().as_millis() as u64;

        // Store page content in variables
        let content = page.content().await.unwrap_or_default();
        self.variables.set_data("SOURCE", content.clone());
        let current_url = page.url().await
            .map(|u| u.map(|u| u.to_string()).unwrap_or_default())
            .unwrap_or_default();
        self.variables.set_data("ADDRESS", current_url.clone());

        // Populate BlockResult with response info so ResponseViewer can display page HTML
        if let Some(last) = self.block_results.last_mut() {
            last.response = Some(ResponseInfo {
                status_code: 200,
                headers: std::collections::HashMap::new(),
                body: content.clone(),
                final_url: current_url.clone(),
                cookies: std::collections::HashMap::new(),
                timing_ms: nav_elapsed,
            });
        }

        // Add to network log so NetworkViewer shows browser navigations
        let cookies_sent: Vec<(String, String)> = if !settings.custom_cookies.is_empty() {
            self.variables.interpolate(&settings.custom_cookies).lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .filter_map(|l| l.split_once('=').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
                .collect()
        } else {
            Vec::new()
        };
        self.network_log.push(NetworkEntry {
            block_id: block_id_for_nav,
            block_label: block_label_for_nav,
            method: "GET".to_string(),
            url: url.clone(),
            status_code: 200,
            timing_ms: nav_elapsed,
            response_size: content.len(),
            cookies_set: Vec::new(),
            cookies_sent,
        });

        self.page = PageHandle(Some(page));
        Ok(())
    }

    pub(super) async fn execute_click_element(&mut self, settings: &ClickElementSettings) -> crate::error::Result<()> {
        let page = self.page.0.as_ref()
            .ok_or_else(|| crate::error::AppError::Pipeline("No page open. Use NavigateTo first.".into()))?;

        let selector = self.variables.interpolate(&settings.selector);
        let element = page.find_element(&selector).await
            .map_err(|e| crate::error::AppError::Pipeline(format!("Element not found '{}': {}", selector, e)))?;

        element.click().await
            .map_err(|e| crate::error::AppError::Pipeline(format!("Click failed: {}", e)))?;

        if settings.wait_for_navigation {
            let _ = page.wait_for_navigation().await;
            if let Ok(content) = page.content().await {
                self.variables.set_data("SOURCE", content);
            }
        }

        Ok(())
    }

    pub(super) async fn execute_type_text(&mut self, settings: &TypeTextSettings) -> crate::error::Result<()> {
        let page = self.page.0.as_ref()
            .ok_or_else(|| crate::error::AppError::Pipeline("No page open. Use NavigateTo first.".into()))?;

        let selector = self.variables.interpolate(&settings.selector);
        let text = self.variables.interpolate(&settings.text);

        let element = page.find_element(&selector).await
            .map_err(|e| crate::error::AppError::Pipeline(format!("Element not found '{}': {}", selector, e)))?;

        if settings.clear_first {
            element.click().await
                .map_err(|e| crate::error::AppError::Pipeline(format!("Click for clear failed: {}", e)))?;
            // Select all and delete to clear
            page.execute(chromiumoxide::cdp::browser_protocol::input::DispatchKeyEventParams::builder()
                .r#type(chromiumoxide::cdp::browser_protocol::input::DispatchKeyEventType::KeyDown)
                .key("a".to_string())
                .modifiers(2) // Ctrl
                .build().unwrap()
            ).await.ok();
            page.execute(chromiumoxide::cdp::browser_protocol::input::DispatchKeyEventParams::builder()
                .r#type(chromiumoxide::cdp::browser_protocol::input::DispatchKeyEventType::KeyDown)
                .key("Backspace".to_string())
                .build().unwrap()
            ).await.ok();
        }

        element.type_str(&text).await
            .map_err(|e| crate::error::AppError::Pipeline(format!("Type text failed: {}", e)))?;

        Ok(())
    }

    pub(super) async fn execute_wait_for_element(&mut self, settings: &WaitForElementSettings) -> crate::error::Result<()> {
        let page = self.page.0.as_ref()
            .ok_or_else(|| crate::error::AppError::Pipeline("No page open. Use NavigateTo first.".into()))?;

        let selector = self.variables.interpolate(&settings.selector);
        let timeout = std::time::Duration::from_millis(settings.timeout_ms);
        let start = std::time::Instant::now();

        loop {
            match page.find_element(&selector).await {
                Ok(_element) => {
                    // Element found - check state if needed
                    match settings.state.as_str() {
                        "visible" | "attached" => break,
                        _ => break,
                    }
                }
                Err(_) => {
                    if start.elapsed() > timeout {
                        return Err(crate::error::AppError::Pipeline(
                            format!("Timeout waiting for element '{}' ({}ms)", selector, settings.timeout_ms)
                        ));
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
            }
        }

        Ok(())
    }

    pub(super) async fn execute_get_element_text(&mut self, settings: &GetElementTextSettings) -> crate::error::Result<()> {
        let page = self.page.0.as_ref()
            .ok_or_else(|| crate::error::AppError::Pipeline("No page open. Use NavigateTo first.".into()))?;

        let selector = self.variables.interpolate(&settings.selector);
        let element = page.find_element(&selector).await
            .map_err(|e| crate::error::AppError::Pipeline(format!("Element not found '{}': {}", selector, e)))?;

        let value = if settings.attribute.is_empty() || settings.attribute == "innerText" {
            element.inner_text().await
                .map_err(|e| crate::error::AppError::Pipeline(format!("Get text failed: {}", e)))?
                .unwrap_or_default()
        } else {
            element.attribute(&settings.attribute).await
                .map_err(|e| crate::error::AppError::Pipeline(format!("Get attribute failed: {}", e)))?
                .unwrap_or_default()
        };

        self.variables.set_user(&settings.output_var, value, settings.capture);
        Ok(())
    }

    pub(super) async fn execute_screenshot(&mut self, settings: &ScreenshotSettings) -> crate::error::Result<()> {
        let page = self.page.0.as_ref()
            .ok_or_else(|| crate::error::AppError::Pipeline("No page open. Use NavigateTo first.".into()))?;

        let bytes = if !settings.selector.is_empty() {
            let selector = self.variables.interpolate(&settings.selector);
            let element = page.find_element(&selector).await
                .map_err(|e| crate::error::AppError::Pipeline(format!("Element not found: {}", e)))?;
            element.screenshot(chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Png).await
                .map_err(|e| crate::error::AppError::Pipeline(format!("Screenshot failed: {}", e)))?
        } else {
            page.screenshot(
                chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams::builder()
                    .format(chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Png)
                    .build()
            ).await
                .map_err(|e| crate::error::AppError::Pipeline(format!("Screenshot failed: {}", e)))?
        };

        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        self.variables.set_user(&settings.output_var, b64, false);
        Ok(())
    }

    pub(super) async fn execute_js(&mut self, settings: &ExecuteJsSettings) -> crate::error::Result<()> {
        let page = self.page.0.as_ref()
            .ok_or_else(|| crate::error::AppError::Pipeline("No page open. Use NavigateTo first.".into()))?;

        let code = self.variables.interpolate(&settings.code);
        let result = page.evaluate_expression(&code).await
            .map_err(|e| crate::error::AppError::Pipeline(format!("JS execution failed: {}", e)))?;

        let value = match result.value() {
            Some(v) => match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            },
            None => String::new(),
        };

        self.variables.set_user(&settings.output_var, value, settings.capture);
        Ok(())
    }
}
