use serde::{Deserialize, Serialize};
use uuid::Uuid;
use futures::StreamExt;

use super::block::*;
use super::variable::VariableStore;
use super::BotStatus;
use super::random_data;
use super::tls_profiles::TLS_PROFILES;
use super::datadome;
use crate::sidecar::protocol::{SidecarRequest, SidecarResponse};

// ── Browser handle wrappers (not serializable/cloneable) ──

#[derive(Default)]
pub struct BrowserHandle(pub Option<chromiumoxide::Browser>);

#[derive(Default)]
pub struct PageHandle(pub Option<chromiumoxide::Page>);

impl Clone for BrowserHandle {
    fn clone(&self) -> Self { Self(None) }
}
impl Clone for PageHandle {
    fn clone(&self) -> Self { Self(None) }
}
impl std::fmt::Debug for BrowserHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "BrowserHandle({})", if self.0.is_some() { "active" } else { "none" })
    }
}
impl std::fmt::Debug for PageHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PageHandle({})", if self.0.is_some() { "active" } else { "none" })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub variables: VariableStore,
    pub status: BotStatus,
    pub session_id: String,
    pub proxy: Option<String>,
    pub log: Vec<LogEntry>,
    pub block_results: Vec<BlockResult>,
    pub network_log: Vec<NetworkEntry>,
    #[serde(skip)]
    pub browser: BrowserHandle,
    #[serde(skip)]
    pub page: PageHandle,
    #[serde(skip)]
    pub override_ja3: Option<String>,
    #[serde(skip)]
    pub override_http2fp: Option<String>,
    #[serde(skip)]
    pub plugin_manager: Option<std::sync::Arc<crate::plugin::manager::PluginManager>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEntry {
    pub block_id: Uuid,
    pub block_label: String,
    pub method: String,
    pub url: String,
    pub status_code: u16,
    pub timing_ms: u64,
    pub response_size: usize,
    pub cookies_set: Vec<(String, String)>,
    pub cookies_sent: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp_ms: u64,
    pub block_id: Uuid,
    pub block_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResult {
    pub block_id: Uuid,
    pub block_label: String,
    pub block_type: BlockType,
    pub success: bool,
    pub timing_ms: u64,
    pub variables_after: std::collections::HashMap<String, String>,
    pub log_message: String,
    // HTTP-specific
    pub request: Option<RequestInfo>,
    pub response: Option<ResponseInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInfo {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseInfo {
    pub status_code: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
    pub final_url: String,
    pub cookies: std::collections::HashMap<String, String>,
    pub timing_ms: u64,
}

impl ExecutionContext {
    pub fn new(session_id: String) -> Self {
        Self {
            variables: VariableStore::new(),
            status: BotStatus::None,
            session_id,
            proxy: None,
            log: Vec::new(),
            block_results: Vec::new(),
            network_log: Vec::new(),
            browser: BrowserHandle(None),
            page: PageHandle(None),
            override_ja3: None,
            override_http2fp: None,
            plugin_manager: None,
        }
    }

    /// Execute a list of blocks sequentially.
    /// Returns after all blocks are run or status changes to non-None.
    pub fn execute_blocks<'a>(
        &'a mut self,
        blocks: &'a [Block],
        sidecar_tx: &'a tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::error::Result<()>> + Send + 'a>> {
        Box::pin(async move {
        for block in blocks {
            if block.disabled {
                continue;
            }

            // Push BlockResult BEFORE execution so execute_http_request's
            // last_mut() points to the correct entry for this block
            self.block_results.push(BlockResult {
                block_id: block.id,
                block_label: block.label.clone(),
                block_type: block.block_type,
                success: true,
                timing_ms: 0,
                variables_after: std::collections::HashMap::new(),
                log_message: String::new(),
                request: None,
                response: None,
            });

            let start = std::time::Instant::now();
            let result = self.execute_block(block, sidecar_tx).await;
            let elapsed = start.elapsed().as_millis() as u64;

            // Update the block result with timing and variables
            if let Some(br) = self.block_results.last_mut() {
                br.timing_ms = elapsed;
                br.variables_after = self.variables.snapshot();
            }

            match result {
                Ok(()) => {
                    let msg = self.block_execution_log(block);
                    if let Some(br) = self.block_results.last_mut() {
                        br.log_message = msg.clone();
                    }
                    if !msg.is_empty() {
                        self.log.push(LogEntry {
                            timestamp_ms: elapsed_ms(),
                            block_id: block.id,
                            block_label: block.label.clone(),
                            message: msg,
                        });
                    }
                }
                Err(e) if block.safe_mode => {
                    let msg = format!("Error (safe mode): {}", e);
                    if let Some(br) = self.block_results.last_mut() {
                        br.success = false;
                        br.log_message = msg.clone();
                    }
                    self.log.push(LogEntry {
                        timestamp_ms: elapsed_ms(),
                        block_id: block.id,
                        block_label: block.label.clone(),
                        message: msg,
                    });
                }
                Err(e) => {
                    self.status = BotStatus::Error;
                    let msg = format!("Error: {}", e);
                    if let Some(br) = self.block_results.last_mut() {
                        br.success = false;
                        br.log_message = msg.clone();
                    }
                    self.log.push(LogEntry {
                        timestamp_ms: elapsed_ms(),
                        block_id: block.id,
                        block_label: block.label.clone(),
                        message: msg.clone(),
                    });
                    return Err(e);
                }
            }

            // Stop if status changed to non-None
            if self.status != BotStatus::None {
                break;
            }
        }
        Ok(())
        })
    }

    async fn execute_block(
        &mut self,
        block: &Block,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        match &block.settings {
            BlockSettings::HttpRequest(settings) => {
                self.execute_http_request(block, settings, sidecar_tx).await
            }
            BlockSettings::ParseLR(settings) => {
                self.execute_parse_lr(settings)
            }
            BlockSettings::ParseRegex(settings) => {
                self.execute_parse_regex(settings)
            }
            BlockSettings::ParseJSON(settings) => {
                self.execute_parse_json(settings)
            }
            BlockSettings::ParseCookie(settings) => {
                self.execute_parse_cookie(settings)
            }
            BlockSettings::KeyCheck(settings) => {
                self.execute_keycheck(settings)
            }
            BlockSettings::StringFunction(settings) => {
                self.execute_string_function(settings)
            }
            BlockSettings::CryptoFunction(settings) => {
                self.execute_crypto_function(settings)
            }
            BlockSettings::Delay(settings) => {
                self.execute_delay(settings).await
            }
            BlockSettings::SetVariable(settings) => {
                let value = self.variables.interpolate(&settings.value);
                self.variables.set_user(&settings.name, value, settings.capture);
                Ok(())
            }
            BlockSettings::Log(settings) => {
                let message = self.variables.interpolate(&settings.message);
                self.log.push(LogEntry {
                    timestamp_ms: elapsed_ms(),
                    block_id: block.id,
                    block_label: block.label.clone(),
                    message,
                });
                Ok(())
            }
            BlockSettings::ClearCookies => {
                let req = SidecarRequest {
                    id: Uuid::new_v4().to_string(),
                    action: "clear_cookies".into(),
                    session: self.session_id.clone(),
                    method: None, url: None, headers: None, body: None,
                    timeout: None, proxy: None, browser: None,
                    ja3: None, http2fp: None, follow_redirects: None, max_redirects: None,
                };
                let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
                let _ = sidecar_tx.send((req, resp_tx)).await;
                let _ = resp_rx.await;
                Ok(())
            }
            BlockSettings::IfElse(settings) => {
                let cond = self.evaluate_condition(&settings.condition);
                let branch = if cond { &settings.true_blocks } else { &settings.false_blocks };
                self.execute_blocks(branch, sidecar_tx).await
            }
            BlockSettings::Loop(settings) => {
                self.execute_loop(block, settings, sidecar_tx).await
            }
            BlockSettings::DateFunction(settings) => {
                self.execute_date_function(settings)
            }
            BlockSettings::CaseSwitch(settings) => {
                self.execute_case_switch(settings)
            }
            BlockSettings::ListFunction(settings) => {
                self.execute_list_function(settings)
            }
            BlockSettings::ConversionFunction(settings) => {
                self.execute_conversion_function(settings)
            }
            BlockSettings::CookieContainer(settings) => {
                self.execute_cookie_container(settings)
            }
            BlockSettings::Script(_settings) => {
                // Script execution is not yet supported in debug mode
                Ok(())
            }
            // Browser automation
            BlockSettings::BrowserOpen(settings) => {
                self.execute_browser_open(settings).await
            }
            BlockSettings::NavigateTo(settings) => {
                self.execute_navigate_to(settings, block.id, block.label.clone()).await
            }
            BlockSettings::ClickElement(settings) => {
                self.execute_click_element(settings).await
            }
            BlockSettings::TypeText(settings) => {
                self.execute_type_text(settings).await
            }
            BlockSettings::WaitForElement(settings) => {
                self.execute_wait_for_element(settings).await
            }
            BlockSettings::GetElementText(settings) => {
                self.execute_get_element_text(settings).await
            }
            BlockSettings::Screenshot(settings) => {
                self.execute_screenshot(settings).await
            }
            BlockSettings::ExecuteJs(settings) => {
                self.execute_js(settings).await
            }
            // Networking (send via sidecar like HTTP)
            BlockSettings::Webhook(settings) => {
                self.execute_webhook(settings, sidecar_tx).await
            }
            // New blocks
            BlockSettings::RandomUserAgent(settings) => {
                self.execute_random_user_agent(settings)
            }
            BlockSettings::OcrCaptcha(settings) => {
                self.execute_ocr_captcha(settings).await
            }
            BlockSettings::RecaptchaInvisible(settings) => {
                self.execute_recaptcha_invisible(settings, sidecar_tx).await
            }
            BlockSettings::XacfSensor(settings) => {
                self.execute_xacf_sensor(settings)
            }
            BlockSettings::RandomData(settings) => {
                self.execute_random_data(settings)
            }
            BlockSettings::DataDomeSensor(settings) => {
                self.execute_datadome_sensor(settings)
            }
            BlockSettings::Plugin(settings) => {
                self.execute_plugin_block(settings)
            }
            BlockSettings::Group(settings) => {
                // Execute child blocks sequentially (like a mini-pipeline)
                self.execute_blocks(&settings.blocks, sidecar_tx).await
            }
            BlockSettings::ParseCSS(settings) => {
                self.execute_parse_css(settings)
            }
            BlockSettings::ParseXPath(settings) => {
                self.execute_parse_xpath(settings)
            }
            // Protocol blocks
            BlockSettings::TcpRequest(settings) => {
                self.execute_tcp_request(block, settings).await
            }
            BlockSettings::UdpRequest(settings) => {
                self.execute_udp_request(block, settings).await
            }
            BlockSettings::FtpRequest(settings) => {
                self.execute_ftp_request(block, settings).await
            }
            BlockSettings::SshRequest(settings) => {
                self.execute_ssh_request(block, settings).await
            }
            BlockSettings::ImapRequest(settings) => {
                self.execute_imap_request(block, settings).await
            }
            BlockSettings::SmtpRequest(settings) => {
                self.execute_smtp_request(block, settings).await
            }
            BlockSettings::PopRequest(settings) => {
                self.execute_pop_request(block, settings).await
            }
            BlockSettings::WebSocket(_settings) => {
                Err(crate::error::AppError::Pipeline("WebSocket requires tokio-tungstenite (not yet added)".into()))
            }
            // Bypass blocks
            BlockSettings::CaptchaSolver(settings) => {
                self.execute_captcha_solver(settings, sidecar_tx).await
            }
            BlockSettings::CloudflareBypass(settings) => {
                self.execute_cloudflare_bypass(settings, sidecar_tx).await
            }
            BlockSettings::LaravelCsrf(settings) => {
                self.execute_laravel_csrf(settings, sidecar_tx).await
            }
        }
    }

    async fn execute_http_request(
        &mut self,
        block: &Block,
        settings: &HttpRequestSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        let url = self.variables.interpolate(&settings.url);
        let body = self.variables.interpolate(&settings.body);
        let mut headers: Vec<Vec<String>> = settings.headers.iter()
            .map(|(k, v)| vec![
                self.variables.interpolate(k),
                self.variables.interpolate(v),
            ])
            .collect();

        // Inject custom cookies as Cookie header
        if !settings.custom_cookies.is_empty() {
            let cookie_str = self.variables.interpolate(&settings.custom_cookies);
            let cookies: Vec<String> = cookie_str.lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect();
            if !cookies.is_empty() {
                headers.push(vec!["Cookie".into(), cookies.join("; ")]);
            }
        }

        let req = SidecarRequest {
            id: Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some(settings.method.clone()),
            url: Some(url.clone()),
            headers: Some(headers.clone()),
            body: Some(body.clone()),
            timeout: Some(settings.timeout_ms as i64),
            proxy: self.proxy.clone(),
            browser: None,
            ja3: self.override_ja3.clone(),
            http2fp: self.override_http2fp.clone(),
            follow_redirects: Some(settings.follow_redirects),
            max_redirects: Some(settings.max_redirects as i64),
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("Failed to send request to sidecar".into()))?;

        let resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("Sidecar response channel closed".into()))?;

        if let Some(ref err) = resp.error {
            if !err.is_empty() {
                return Err(crate::error::AppError::Sidecar(err.clone()));
            }
        }

        // Store response data in named variables
        let var_prefix = if settings.response_var.is_empty() { "SOURCE" } else { &settings.response_var };
        self.variables.set_data(var_prefix, resp.body.clone());
        self.variables.set_data(&format!("{}.STATUS", var_prefix), resp.status.to_string());
        self.variables.set_data(&format!("{}.URL", var_prefix), resp.final_url.clone());
        if let Some(ref hdrs) = resp.headers {
            let hdr_str = serde_json::to_string(hdrs).unwrap_or_default();
            self.variables.set_data(&format!("{}.HEADERS", var_prefix), hdr_str);
        }
        if let Some(ref cookies) = resp.cookies {
            let cookie_str = serde_json::to_string(cookies).unwrap_or_default();
            self.variables.set_data(&format!("{}.COOKIES", var_prefix), cookie_str);
        }
        // Backward compat: always set RESPONSECODE and ADDRESS
        self.variables.set_data("RESPONSECODE", resp.status.to_string());
        self.variables.set_data("ADDRESS", resp.final_url.clone());

        // Update block result with request/response info
        if let Some(last) = self.block_results.last_mut() {
            last.request = Some(RequestInfo {
                method: settings.method.clone(),
                url: url.clone(),
                headers: settings.headers.clone(),
                body: body.clone(),
            });
            last.response = Some(ResponseInfo {
                status_code: resp.status as u16,
                headers: resp.headers.clone().unwrap_or_default(),
                body: resp.body.clone(),
                final_url: resp.final_url.clone(),
                cookies: resp.cookies.clone().unwrap_or_default(),
                timing_ms: resp.timing_ms as u64,
            });
        }

        // Add to network log
        let cookies_sent: Vec<(String, String)> = settings.custom_cookies.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .filter_map(|l| l.split_once('=').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
            .collect();
        let cookies_set: Vec<(String, String)> = resp.cookies.as_ref()
            .map(|c| c.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        self.network_log.push(NetworkEntry {
            block_id: block.id,
            block_label: block.label.clone(),
            method: settings.method.clone(),
            url,
            status_code: resp.status as u16,
            timing_ms: resp.timing_ms as u64,
            response_size: resp.body.len(),
            cookies_set,
            cookies_sent,
        });

        Ok(())
    }

    fn execute_parse_lr(&mut self, settings: &ParseLRSettings) -> crate::error::Result<()> {
        let source = self.variables.get(&settings.input_var).unwrap_or_default();
        let left = self.variables.interpolate(&settings.left);
        let right = self.variables.interpolate(&settings.right);

        if settings.recursive {
            let mut results = Vec::new();
            let mut search_from = 0;
            while let Some(start) = source[search_from..].find(&left) {
                let abs_start = search_from + start + left.len();
                if let Some(end) = source[abs_start..].find(&right) {
                    results.push(source[abs_start..abs_start + end].to_string());
                    search_from = abs_start + end + right.len();
                } else {
                    break;
                }
            }
            let value = results.join(", ");
            self.variables.set_user(&settings.output_var, value, settings.capture);
        } else {
            let value = if let Some(start) = source.find(&left) {
                let after = start + left.len();
                if let Some(end) = source[after..].find(&right) {
                    source[after..after + end].to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            self.variables.set_user(&settings.output_var, value, settings.capture);
        }

        Ok(())
    }

    fn execute_parse_regex(&mut self, settings: &ParseRegexSettings) -> crate::error::Result<()> {
        let source = self.variables.get(&settings.input_var).unwrap_or_default();
        let pattern = self.variables.interpolate(&settings.pattern);
        let re = regex::Regex::new(&pattern)?;

        if let Some(caps) = re.captures(&source) {
            let mut output = settings.output_format.clone();
            for i in 0..caps.len() {
                let group_val = caps.get(i).map(|m| m.as_str()).unwrap_or("");
                output = output.replace(&format!("${}", i), group_val);
            }
            self.variables.set_user(&settings.output_var, output, settings.capture);
        }

        Ok(())
    }

    fn execute_parse_json(&mut self, settings: &ParseJSONSettings) -> crate::error::Result<()> {
        let source = self.variables.get(&settings.input_var).unwrap_or_default();
        let path = self.variables.interpolate(&settings.json_path);

        let json: serde_json::Value = serde_json::from_str(&source)
            .map_err(|e| crate::error::AppError::Pipeline(format!("Invalid JSON: {}", e)))?;

        // Convert dot notation to JSON pointer: "user.token" → "/user/token"
        let pointer = if path.starts_with('/') {
            path.clone()
        } else {
            format!("/{}", path.replace('.', "/"))
        };

        let value = json.pointer(&pointer)
            .map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .unwrap_or_default();

        self.variables.set_user(&settings.output_var, value, settings.capture);
        Ok(())
    }

    fn execute_parse_cookie(&mut self, settings: &ParseCookieSettings) -> crate::error::Result<()> {
        let source = self.variables.get(&settings.input_var).unwrap_or_default();
        let cookie_name = self.variables.interpolate(&settings.cookie_name);

        // Source is expected to be a JSON object {"name":"value",...}
        let value = if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, String>>(&source) {
            map.get(&cookie_name).cloned().unwrap_or_default()
        } else {
            // Fallback: try parsing as "name=value; name2=value2" cookie header string
            source.split(';')
                .filter_map(|pair| {
                    let pair = pair.trim();
                    let eq = pair.find('=')?;
                    Some((&pair[..eq], &pair[eq + 1..]))
                })
                .find(|(name, _)| *name == cookie_name)
                .map(|(_, v)| v.to_string())
                .unwrap_or_default()
        };

        self.variables.set_user(&settings.output_var, value, settings.capture);
        Ok(())
    }

    fn execute_parse_css(&mut self, settings: &ParseCSSSettings) -> crate::error::Result<()> {
        let source = self.variables.get(&settings.input_var).unwrap_or_default();
        let selector_str = self.variables.interpolate(&settings.selector);
        let attribute = self.variables.interpolate(&settings.attribute);

        let document = scraper::Html::parse_document(&source);
        let selector = scraper::Selector::parse(&selector_str)
            .map_err(|e| crate::error::AppError::Pipeline(format!("Invalid CSS selector '{}': {:?}", selector_str, e)))?;

        let elements: Vec<_> = document.select(&selector).collect();
        let value = if elements.is_empty() {
            String::new()
        } else {
            let idx = settings.index as usize;
            if idx < elements.len() {
                let el = &elements[idx];
                if attribute.is_empty() || attribute == "text" || attribute == "innerText" {
                    el.text().collect::<Vec<_>>().join("")
                } else if attribute == "innerHTML" || attribute == "html" {
                    el.inner_html()
                } else if attribute == "outerHTML" {
                    el.html()
                } else {
                    el.value().attr(&attribute).unwrap_or("").to_string()
                }
            } else {
                String::new()
            }
        };

        self.variables.set_user(&settings.output_var, value.trim().to_string(), settings.capture);
        Ok(())
    }

    fn execute_parse_xpath(&mut self, settings: &ParseXPathSettings) -> crate::error::Result<()> {
        let source = self.variables.get(&settings.input_var).unwrap_or_default();
        let xpath_str = self.variables.interpolate(&settings.xpath);

        let package = sxd_document::parser::parse(&source);
        let value = match package {
            Ok(package) => {
                let doc = package.as_document();
                let factory = sxd_xpath::Factory::new();
                match factory.build(&xpath_str) {
                    Ok(Some(xpath)) => {
                        let ctx = sxd_xpath::Context::new();
                        match xpath.evaluate(&ctx, doc.root()) {
                            Ok(val) => {
                                match val {
                                    sxd_xpath::Value::String(s) => s,
                                    sxd_xpath::Value::Number(n) => n.to_string(),
                                    sxd_xpath::Value::Boolean(b) => b.to_string(),
                                    sxd_xpath::Value::Nodeset(ns) => {
                                        ns.iter()
                                            .map(|node| node.string_value())
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    }
                                }
                            }
                            Err(_) => {
                                String::new()
                            }
                        }
                    }
                    Ok(None) => String::new(),
                    Err(e) => {
                        return Err(crate::error::AppError::Pipeline(
                            format!("Invalid XPath '{}': {:?}", xpath_str, e),
                        ));
                    }
                }
            }
            Err(_) => {
                // If the source isn't valid XML, try wrapping in a root element
                let wrapped = format!("<root>{}</root>", source);
                match sxd_document::parser::parse(&wrapped) {
                    Ok(package) => {
                        let doc = package.as_document();
                        let factory = sxd_xpath::Factory::new();
                        match factory.build(&xpath_str) {
                            Ok(Some(xpath)) => {
                                let ctx = sxd_xpath::Context::new();
                                match xpath.evaluate(&ctx, doc.root()) {
                                    Ok(val) => match val {
                                        sxd_xpath::Value::String(s) => s,
                                        sxd_xpath::Value::Number(n) => n.to_string(),
                                        sxd_xpath::Value::Boolean(b) => b.to_string(),
                                        sxd_xpath::Value::Nodeset(ns) => {
                                            ns.iter().map(|n| n.string_value()).collect::<Vec<_>>().join(", ")
                                        }
                                    },
                                    Err(_) => String::new(),
                                }
                            }
                            _ => String::new(),
                        }
                    }
                    Err(_) => String::new(),
                }
            }
        };

        self.variables.set_user(&settings.output_var, value.trim().to_string(), settings.capture);
        Ok(())
    }

    fn execute_keycheck(&mut self, settings: &KeyCheckSettings) -> crate::error::Result<()> {
        for keychain in &settings.keychains {
            let all_match = keychain.conditions.iter().all(|cond| self.evaluate_condition(cond));
            if all_match {
                self.status = keychain.result;
                break;
            }
        }
        Ok(())
    }

    fn evaluate_condition(&self, cond: &KeyCondition) -> bool {
        let source_val = self.variables.get(&cond.source).unwrap_or_default();
        let target = self.variables.interpolate(&cond.value);

        match cond.comparison {
            Comparison::Contains => source_val.contains(&target),
            Comparison::NotContains => !source_val.contains(&target),
            Comparison::EqualTo => source_val == target,
            Comparison::NotEqualTo => source_val != target,
            Comparison::MatchesRegex => {
                regex::Regex::new(&target).map(|re| re.is_match(&source_val)).unwrap_or(false)
            }
            Comparison::GreaterThan => {
                source_val.parse::<f64>().unwrap_or(0.0) > target.parse::<f64>().unwrap_or(0.0)
            }
            Comparison::LessThan => {
                source_val.parse::<f64>().unwrap_or(0.0) < target.parse::<f64>().unwrap_or(0.0)
            }
            Comparison::Exists => !source_val.is_empty(),
            Comparison::NotExists => source_val.is_empty(),
        }
    }

    fn execute_string_function(&mut self, settings: &StringFunctionSettings) -> crate::error::Result<()> {
        let input = self.variables.get(&settings.input_var).unwrap_or_default();
        let param1 = self.variables.interpolate(&settings.param1);
        let param2 = self.variables.interpolate(&settings.param2);

        let result = match settings.function_type {
            StringFnType::Replace => input.replace(&param1, &param2),
            StringFnType::Substring => {
                let start: usize = param1.parse().unwrap_or(0);
                let len: usize = param2.parse().unwrap_or(input.len());
                input.chars().skip(start).take(len).collect()
            }
            StringFnType::Trim => input.trim().to_string(),
            StringFnType::ToUpper => input.to_uppercase(),
            StringFnType::ToLower => input.to_lowercase(),
            StringFnType::URLEncode => urlencoding(&input),
            StringFnType::URLDecode => urldecoding(&input),
            StringFnType::Base64Encode => {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(input.as_bytes())
            }
            StringFnType::Base64Decode => {
                use base64::Engine;
                match base64::engine::general_purpose::STANDARD.decode(input.as_bytes()) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                    Err(_) => String::new(),
                }
            }
            StringFnType::Split => {
                let parts: Vec<String> = input.split(&param1).map(|s| s.to_string()).collect();
                serde_json::to_string(&parts).unwrap_or_default()
            }
            StringFnType::RandomString => {
                let len: usize = param1.parse().unwrap_or(16);
                use rand::Rng;
                let mut rng = rand::thread_rng();
                (0..len).map(|_| {
                    let idx = rng.gen_range(0..36);
                    if idx < 10 { (b'0' + idx) as char } else { (b'a' + idx - 10) as char }
                }).collect()
            }
            StringFnType::Reverse => input.chars().rev().collect(),
            StringFnType::Length => input.len().to_string(),
            _ => input,
        };

        self.variables.set_user(&settings.output_var, result, settings.capture);
        Ok(())
    }

    fn execute_crypto_function(&mut self, settings: &CryptoFunctionSettings) -> crate::error::Result<()> {
        let input = self.variables.get(&settings.input_var).unwrap_or_default();
        let key = self.variables.interpolate(&settings.key);

        let result = match settings.function_type {
            CryptoFnType::MD5 => {
                use md5::Digest;
                format!("{:x}", md5::Md5::digest(input.as_bytes()))
            }
            CryptoFnType::SHA1 => {
                use sha1::Digest;
                format!("{:x}", sha1::Sha1::digest(input.as_bytes()))
            }
            CryptoFnType::SHA256 => {
                use sha2::{Sha256, Digest};
                format!("{:x}", Sha256::digest(input.as_bytes()))
            }
            CryptoFnType::SHA384 => {
                use sha2::{Sha384, Digest};
                format!("{:x}", Sha384::digest(input.as_bytes()))
            }
            CryptoFnType::SHA512 => {
                use sha2::{Sha512, Digest};
                format!("{:x}", Sha512::digest(input.as_bytes()))
            }
            CryptoFnType::CRC32 => {
                let crc = crc32fast::hash(input.as_bytes());
                format!("{:08x}", crc)
            }
            CryptoFnType::HMACSHA256 => {
                use hmac::{Hmac, Mac};
                type HmacSha256 = Hmac<sha2::Sha256>;
                let mut mac = HmacSha256::new_from_slice(key.as_bytes())
                    .map_err(|e| crate::error::AppError::Pipeline(format!("HMAC key error: {}", e)))?;
                mac.update(input.as_bytes());
                format!("{:x}", mac.finalize().into_bytes())
            }
            CryptoFnType::HMACSHA512 => {
                use hmac::{Hmac, Mac};
                type HmacSha512 = Hmac<sha2::Sha512>;
                let mut mac = HmacSha512::new_from_slice(key.as_bytes())
                    .map_err(|e| crate::error::AppError::Pipeline(format!("HMAC key error: {}", e)))?;
                mac.update(input.as_bytes());
                format!("{:x}", mac.finalize().into_bytes())
            }
            CryptoFnType::HMACMD5 => {
                use hmac::{Hmac, Mac};
                type HmacMd5 = Hmac<md5::Md5>;
                let mut mac = HmacMd5::new_from_slice(key.as_bytes())
                    .map_err(|e| crate::error::AppError::Pipeline(format!("HMAC key error: {}", e)))?;
                mac.update(input.as_bytes());
                format!("{:x}", mac.finalize().into_bytes())
            }
            CryptoFnType::BCryptHash => {
                let cost = key.parse::<u32>().unwrap_or(12);
                bcrypt::hash(input, cost)
                    .map_err(|e| crate::error::AppError::Pipeline(format!("BCrypt hash error: {}", e)))?
            }
            CryptoFnType::BCryptVerify => {
                // key = the hash to verify against
                let valid = bcrypt::verify(input, &key)
                    .map_err(|e| crate::error::AppError::Pipeline(format!("BCrypt verify error: {}", e)))?;
                valid.to_string()
            }
            CryptoFnType::AESEncrypt => {
                use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
                use aes_gcm::aead::generic_array::GenericArray;

                let key_bytes = if key.len() == 64 {
                    // Hex-encoded 32-byte key
                    (0..key.len()).step_by(2)
                        .filter_map(|i| u8::from_str_radix(&key[i..i+2], 16).ok())
                        .collect::<Vec<u8>>()
                } else {
                    // Pad/truncate to 32 bytes
                    let mut k = key.as_bytes().to_vec();
                    k.resize(32, 0);
                    k
                };
                let cipher = Aes256Gcm::new(GenericArray::from_slice(&key_bytes));
                let nonce_bytes: [u8; 12] = rand::random();
                let nonce = GenericArray::from_slice(&nonce_bytes);
                let ciphertext = cipher.encrypt(nonce, input.as_bytes())
                    .map_err(|e| crate::error::AppError::Pipeline(format!("AES encrypt error: {}", e)))?;
                // Output: hex(nonce) + ":" + hex(ciphertext)
                let nonce_hex: String = nonce_bytes.iter().map(|b| format!("{:02x}", b)).collect();
                let ct_hex: String = ciphertext.iter().map(|b| format!("{:02x}", b)).collect();
                format!("{}:{}", nonce_hex, ct_hex)
            }
            CryptoFnType::AESDecrypt => {
                use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
                use aes_gcm::aead::generic_array::GenericArray;

                let key_bytes = if key.len() == 64 {
                    (0..key.len()).step_by(2)
                        .filter_map(|i| u8::from_str_radix(&key[i..i+2], 16).ok())
                        .collect::<Vec<u8>>()
                } else {
                    let mut k = key.as_bytes().to_vec();
                    k.resize(32, 0);
                    k
                };
                let cipher = Aes256Gcm::new(GenericArray::from_slice(&key_bytes));
                let parts: Vec<&str> = input.splitn(2, ':').collect();
                if parts.len() != 2 {
                    return Err(crate::error::AppError::Pipeline("AES decrypt: expected nonce:ciphertext format".into()));
                }
                let nonce_bytes: Vec<u8> = (0..parts[0].len()).step_by(2)
                    .filter_map(|i| u8::from_str_radix(&parts[0][i..i+2], 16).ok())
                    .collect();
                let ct_bytes: Vec<u8> = (0..parts[1].len()).step_by(2)
                    .filter_map(|i| u8::from_str_radix(&parts[1][i..i+2], 16).ok())
                    .collect();
                let nonce = GenericArray::from_slice(&nonce_bytes);
                let plaintext = cipher.decrypt(nonce, ct_bytes.as_ref())
                    .map_err(|e| crate::error::AppError::Pipeline(format!("AES decrypt error: {}", e)))?;
                String::from_utf8_lossy(&plaintext).to_string()
            }
            CryptoFnType::Base64Encode => {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(input.as_bytes())
            }
            CryptoFnType::Base64Decode => {
                use base64::Engine;
                match base64::engine::general_purpose::STANDARD.decode(input.as_bytes()) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                    Err(_) => String::new(),
                }
            }
        };

        self.variables.set_user(&settings.output_var, result, settings.capture);
        Ok(())
    }

    async fn execute_delay(&self, settings: &DelaySettings) -> crate::error::Result<()> {
        let ms = if settings.min_ms == settings.max_ms {
            settings.min_ms
        } else {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            rng.gen_range(settings.min_ms..=settings.max_ms)
        };
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
        Ok(())
    }

    async fn execute_loop(
        &mut self,
        _block: &Block,
        settings: &LoopSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        match settings.loop_type {
            LoopType::ForEach => {
                let list_str = self.variables.get(&settings.list_var).unwrap_or_default();
                let items: Vec<String> = serde_json::from_str(&list_str)
                    .unwrap_or_else(|_| vec![list_str]);
                for item in items {
                    self.variables.set_user(&settings.item_var, item, false);
                    self.execute_blocks(&settings.blocks, sidecar_tx).await?;
                    if self.status != BotStatus::None {
                        break;
                    }
                }
            }
            LoopType::Repeat => {
                for _ in 0..settings.count {
                    self.execute_blocks(&settings.blocks, sidecar_tx).await?;
                    if self.status != BotStatus::None {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    // ── Date Function ──

    fn execute_date_function(&mut self, settings: &DateFunctionSettings) -> crate::error::Result<()> {
        let result = match settings.function_type {
            DateFnType::Now => {
                chrono::Local::now().format(&settings.format).to_string()
            }
            DateFnType::UnixTimestamp => {
                chrono::Utc::now().timestamp().to_string()
            }
            DateFnType::UnixToDate => {
                let input = self.variables.get(&settings.input_var).unwrap_or_default();
                let ts: i64 = input.parse().unwrap_or(0);
                if let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) {
                    dt.format(&settings.format).to_string()
                } else {
                    String::new()
                }
            }
            DateFnType::FormatDate => {
                let input = self.variables.get(&settings.input_var).unwrap_or_default();
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&input, "%Y-%m-%d %H:%M:%S") {
                    dt.format(&settings.format).to_string()
                } else {
                    input
                }
            }
            DateFnType::ParseDate => {
                let input = self.variables.get(&settings.input_var).unwrap_or_default();
                let fmt = self.variables.interpolate(&settings.format);
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&input, &fmt) {
                    dt.and_utc().timestamp().to_string()
                } else {
                    String::new()
                }
            }
            DateFnType::AddTime | DateFnType::SubtractTime => {
                let input = self.variables.get(&settings.input_var).unwrap_or_default();
                let ts: i64 = input.parse().unwrap_or_else(|_| chrono::Utc::now().timestamp());
                let amount = settings.amount;
                let delta_secs = match settings.unit.as_str() {
                    "seconds" => amount,
                    "minutes" => amount * 60,
                    "hours" => amount * 3600,
                    "days" => amount * 86400,
                    _ => amount,
                };
                let new_ts = if matches!(settings.function_type, DateFnType::AddTime) {
                    ts + delta_secs
                } else {
                    ts - delta_secs
                };
                if let Some(dt) = chrono::DateTime::from_timestamp(new_ts, 0) {
                    dt.format(&settings.format).to_string()
                } else {
                    new_ts.to_string()
                }
            }
        };
        self.variables.set_user(&settings.output_var, result, settings.capture);
        Ok(())
    }

    // ── Case / Switch ──

    fn execute_case_switch(&mut self, settings: &CaseSwitchSettings) -> crate::error::Result<()> {
        let input = self.variables.get(&settings.input_var).unwrap_or_default();
        let result = settings.cases.iter()
            .find(|c| c.match_value == input)
            .map(|c| self.variables.interpolate(&c.result_value))
            .unwrap_or_else(|| self.variables.interpolate(&settings.default_value));
        self.variables.set_user(&settings.output_var, result, settings.capture);
        Ok(())
    }

    // ── List Function ──

    fn execute_list_function(&mut self, settings: &ListFunctionSettings) -> crate::error::Result<()> {
        let input = self.variables.get(&settings.input_var).unwrap_or_default();
        let param1 = self.variables.interpolate(&settings.param1);

        let items: Vec<String> = serde_json::from_str(&input)
            .unwrap_or_else(|_| vec![input.clone()]);

        let result = match settings.function_type {
            ListFnType::Join => items.join(&param1),
            ListFnType::Sort => {
                let mut sorted = items;
                sorted.sort();
                serde_json::to_string(&sorted).unwrap_or_default()
            }
            ListFnType::Shuffle => {
                use rand::seq::SliceRandom;
                let mut shuffled = items;
                shuffled.shuffle(&mut rand::thread_rng());
                serde_json::to_string(&shuffled).unwrap_or_default()
            }
            ListFnType::Add => {
                let mut list = items;
                list.push(param1);
                serde_json::to_string(&list).unwrap_or_default()
            }
            ListFnType::Remove => {
                let list: Vec<String> = items.into_iter().filter(|i| *i != param1).collect();
                serde_json::to_string(&list).unwrap_or_default()
            }
            ListFnType::Deduplicate => {
                let mut seen = std::collections::HashSet::new();
                let deduped: Vec<String> = items.into_iter().filter(|i| seen.insert(i.clone())).collect();
                serde_json::to_string(&deduped).unwrap_or_default()
            }
            ListFnType::RandomItem => {
                use rand::seq::SliceRandom;
                items.choose(&mut rand::thread_rng()).cloned().unwrap_or_default()
            }
            ListFnType::Length => items.len().to_string(),
        };

        self.variables.set_user(&settings.output_var, result, settings.capture);
        Ok(())
    }

    // ── Conversion Function ──

    fn execute_conversion_function(&mut self, settings: &ConversionFunctionSettings) -> crate::error::Result<()> {
        let input = self.variables.get(&settings.input_var).unwrap_or_default();

        let result = match (settings.from_type.as_str(), settings.to_type.as_str()) {
            ("string", "int") | ("string", "integer") => {
                input.parse::<i64>().map(|v| v.to_string()).unwrap_or_default()
            }
            ("string", "float") | ("string", "double") => {
                input.parse::<f64>().map(|v| v.to_string()).unwrap_or_default()
            }
            ("string", "bool") | ("string", "boolean") => {
                match input.to_lowercase().as_str() {
                    "true" | "1" | "yes" => "true".into(),
                    _ => "false".into(),
                }
            }
            ("int", "string") | ("float", "string") | ("bool", "string") => input,
            ("string", "hex") => {
                input.as_bytes().iter().map(|b| format!("{:02x}", b)).collect()
            }
            ("hex", "string") => {
                let bytes: Vec<u8> = (0..input.len())
                    .step_by(2)
                    .filter_map(|i| u8::from_str_radix(&input[i..i+2], 16).ok())
                    .collect();
                String::from_utf8_lossy(&bytes).to_string()
            }
            _ => input,
        };

        self.variables.set_user(&settings.output_var, result, settings.capture);
        Ok(())
    }

    // ── Cookie Container (OpenBullet-style) ──

    fn execute_cookie_container(&mut self, settings: &CookieContainerSettings) -> crate::error::Result<()> {
        let raw_text = if settings.source_type == "file" {
            let path = self.variables.interpolate(&settings.source);
            std::fs::read_to_string(&path)
                .map_err(|e| crate::error::AppError::Pipeline(format!("Cookie file read error: {}", e)))?
        } else {
            self.variables.interpolate(&settings.source)
        };

        let domain_filter = self.variables.interpolate(&settings.domain);

        // Parse cookies — supports Netscape format (tab-separated) and simple name=value
        let mut cookies: Vec<(String, String)> = Vec::new();
        let mut netscape_lines: Vec<String> = Vec::new();
        let mut seen_keys = std::collections::HashSet::new();

        for line in raw_text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 7 {
                // Netscape format: domain, flag, path, secure, expiry, name, value
                let cookie_domain = parts[0];
                let name = parts[5];
                let value = parts[6];

                // Domain filter
                if !domain_filter.is_empty() && !cookie_domain.contains(&domain_filter) {
                    continue;
                }

                // Deduplicate
                if seen_keys.insert(name.to_string()) {
                    cookies.push((name.to_string(), value.to_string()));
                    netscape_lines.push(line.to_string());
                }
            } else if let Some(eq) = line.find('=') {
                // Simple name=value format
                let name = line[..eq].trim();
                let value = line[eq+1..].trim();
                if seen_keys.insert(name.to_string()) {
                    cookies.push((name.to_string(), value.to_string()));
                }
            }
        }

        // Store as "name=value; name2=value2" format
        let cookie_string = cookies.iter()
            .map(|(n, v)| format!("{}={}", n, v))
            .collect::<Vec<_>>()
            .join("; ");
        self.variables.set_user(&settings.output_var, cookie_string, settings.capture);

        // Optionally store in Netscape format
        if settings.save_netscape && !netscape_lines.is_empty() {
            self.variables.set_user(
                &format!("{}_NETSCAPE", settings.output_var),
                netscape_lines.join("\n"),
                false,
            );
        }

        Ok(())
    }

    // ── Browser Automation ──

    async fn execute_browser_open(&mut self, settings: &BrowserOpenSettings) -> crate::error::Result<()> {
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

        // Spawn CDP event handler in background — must NOT break on errors
        // or all future browser commands fail with "oneshot cancelled"
        tokio::spawn(async move {
            while handler.next().await.is_some() {}
        });

        self.browser = BrowserHandle(Some(browser));
        self.page = PageHandle(None);
        Ok(())
    }

    async fn execute_navigate_to(&mut self, settings: &NavigateToSettings, block_id_for_nav: Uuid, block_label_for_nav: String) -> crate::error::Result<()> {
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

    async fn execute_click_element(&mut self, settings: &ClickElementSettings) -> crate::error::Result<()> {
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

    async fn execute_type_text(&mut self, settings: &TypeTextSettings) -> crate::error::Result<()> {
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

    async fn execute_wait_for_element(&mut self, settings: &WaitForElementSettings) -> crate::error::Result<()> {
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

    async fn execute_get_element_text(&mut self, settings: &GetElementTextSettings) -> crate::error::Result<()> {
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

    async fn execute_screenshot(&mut self, settings: &ScreenshotSettings) -> crate::error::Result<()> {
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

    async fn execute_js(&mut self, settings: &ExecuteJsSettings) -> crate::error::Result<()> {
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

    // ── Webhook (via sidecar HTTP) ──

    async fn execute_webhook(
        &mut self,
        settings: &WebhookSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        let url = self.variables.interpolate(&settings.url);
        let body = self.variables.interpolate(&settings.body_template);
        let mut headers: Vec<Vec<String>> = settings.headers.iter()
            .map(|(k, v)| vec![self.variables.interpolate(k), self.variables.interpolate(v)])
            .collect();

        // Inject custom cookies as Cookie header
        if !settings.custom_cookies.is_empty() {
            let cookie_str = self.variables.interpolate(&settings.custom_cookies);
            let cookies: Vec<String> = cookie_str.lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect();
            if !cookies.is_empty() {
                headers.push(vec!["Cookie".into(), cookies.join("; ")]);
            }
        }

        let req = SidecarRequest {
            id: Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some(settings.method.clone()),
            url: Some(url),
            headers: Some(headers),
            body: Some(body),
            timeout: Some(10000),
            proxy: self.proxy.clone(),
            browser: None,
            ja3: self.override_ja3.clone(),
            http2fp: self.override_http2fp.clone(),
            follow_redirects: Some(true),
            max_redirects: Some(5),
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("Failed to send webhook".into()))?;
        let _resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("Webhook response channel closed".into()))?;
        Ok(())
    }

    // ── Random User Agent ──

    fn execute_random_user_agent(&mut self, settings: &RandomUserAgentSettings) -> crate::error::Result<()> {
        use rand::seq::SliceRandom;

        if settings.match_tls {
            // Use TLS profiles for matched UA + JA3 + HTTP/2 fingerprint
            let filtered: Vec<_> = TLS_PROFILES.iter()
                .filter(|p| {
                    let browser_ok = settings.browser_filter.is_empty()
                        || settings.browser_filter.iter().any(|f| f.eq_ignore_ascii_case(p.browser));
                    let platform_ok = settings.platform_filter.is_empty()
                        || settings.platform_filter.iter().any(|f| f.eq_ignore_ascii_case(p.platform));
                    browser_ok && platform_ok
                })
                .collect();
            if let Some(profile) = filtered.choose(&mut rand::thread_rng()) {
                self.variables.set_user(&settings.output_var, profile.user_agent.to_string(), settings.capture);
                self.override_ja3 = Some(profile.ja3_hash.to_string());
                self.override_http2fp = Some(profile.http2_fingerprint.to_string());
            } else if let Some(profile) = TLS_PROFILES.first() {
                self.variables.set_user(&settings.output_var, profile.user_agent.to_string(), settings.capture);
                self.override_ja3 = Some(profile.ja3_hash.to_string());
                self.override_http2fp = Some(profile.http2_fingerprint.to_string());
            }
            return Ok(());
        }

        let ua = match settings.mode {
            UserAgentMode::CustomList => {
                let list = self.variables.interpolate(&settings.custom_list);
                let items: Vec<&str> = list.lines().filter(|l| !l.trim().is_empty()).collect();
                items.choose(&mut rand::thread_rng())
                    .map(|s| s.to_string())
                    .unwrap_or_default()
            }
            UserAgentMode::Random => {
                let filtered: Vec<&(&str, &str, &str)> = BUILTIN_USER_AGENTS.iter()
                    .filter(|(_, browser, platform)| {
                        let browser_ok = settings.browser_filter.is_empty()
                            || settings.browser_filter.iter().any(|f| f.eq_ignore_ascii_case(browser));
                        let platform_ok = settings.platform_filter.is_empty()
                            || settings.platform_filter.iter().any(|f| f.eq_ignore_ascii_case(platform));
                        browser_ok && platform_ok
                    })
                    .collect();
                filtered.choose(&mut rand::thread_rng())
                    .map(|(ua, _, _)| ua.to_string())
                    .unwrap_or_else(|| BUILTIN_USER_AGENTS[0].0.to_string())
            }
        };

        self.variables.set_user(&settings.output_var, ua, settings.capture);
        Ok(())
    }

    // ── OCR Captcha ──

    async fn execute_ocr_captcha(&mut self, settings: &OcrCaptchaSettings) -> crate::error::Result<()> {
        let input_b64 = self.variables.get(&settings.input_var).unwrap_or_default();

        use base64::Engine;
        let image_bytes = base64::engine::general_purpose::STANDARD.decode(input_b64.as_bytes())
            .map_err(|e| crate::error::AppError::Pipeline(format!("OCR base64 decode error: {}", e)))?;

        // Write to temp file
        let temp_path = std::env::temp_dir().join(format!("reqflow_ocr_{}.png", uuid::Uuid::new_v4()));
        std::fs::write(&temp_path, &image_bytes)
            .map_err(|e| crate::error::AppError::Pipeline(format!("OCR temp file write error: {}", e)))?;

        // Build tesseract args
        let mut args = rusty_tesseract::Args::default();
        args.lang = settings.language.clone();
        args.psm = Some(settings.psm as i32);
        if !settings.whitelist.is_empty() {
            args.config_variables.insert(
                "tessedit_char_whitelist".to_string(),
                settings.whitelist.clone(),
            );
        }

        let image = rusty_tesseract::Image::from_path(&temp_path)
            .map_err(|e| crate::error::AppError::Pipeline(format!("OCR image load error: {}", e)))?;

        let result = rusty_tesseract::image_to_string(&image, &args)
            .map_err(|e| crate::error::AppError::Pipeline(format!("OCR recognition error: {}", e)))?;

        // Cleanup temp file
        let _ = std::fs::remove_file(&temp_path);

        self.variables.set_user(&settings.output_var, result.trim().to_string(), settings.capture);
        Ok(())
    }

    // ── reCAPTCHA Invisible ──

    async fn execute_recaptcha_invisible(
        &mut self,
        settings: &RecaptchaInvisibleSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        let anchor_url = self.variables.interpolate(&settings.anchor_url);
        let reload_url = self.variables.interpolate(&settings.reload_url);
        let user_agent = self.variables.interpolate(&settings.user_agent);

        // Step 1: GET anchor URL to get initial recaptcha-token
        let anchor_req = SidecarRequest {
            id: uuid::Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some("GET".into()),
            url: Some(anchor_url.clone()),
            headers: Some(vec![
                vec!["User-Agent".into(), user_agent.clone()],
            ]),
            body: Some(String::new()),
            timeout: Some(15000),
            proxy: self.proxy.clone(),
            browser: None, ja3: self.override_ja3.clone(), http2fp: self.override_http2fp.clone(),
            follow_redirects: Some(true),
            max_redirects: Some(5),
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((anchor_req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("Failed to send anchor request".into()))?;
        let anchor_resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("Anchor response channel closed".into()))?;

        // Parse recaptcha-token from HTML
        let token = {
            let body = &anchor_resp.body;
            let marker = "recaptcha-token\" value=\"";
            if let Some(start) = body.find(marker) {
                let after = start + marker.len();
                if let Some(end) = body[after..].find('"') {
                    body[after..after + end].to_string()
                } else {
                    return Err(crate::error::AppError::Pipeline("Failed to parse recaptcha-token end".into()));
                }
            } else {
                return Err(crate::error::AppError::Pipeline("recaptcha-token not found in anchor response".into()));
            }
        };

        // Step 2: POST reload URL with token
        let post_body = format!(
            "v={}&reason=q&c={}&k={}&co={}&hl=en&size={}&chr=%5B89%2C64%2C27%5D&vh=13599012192&bg=!q62grYxHRvVxjUIjSFNd0mlr",
            self.variables.interpolate(&settings.v),
            token,
            self.variables.interpolate(&settings.sitekey),
            self.variables.interpolate(&settings.co),
            self.variables.interpolate(&settings.size),
        );

        let reload_req = SidecarRequest {
            id: uuid::Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some("POST".into()),
            url: Some(reload_url),
            headers: Some(vec![
                vec!["User-Agent".into(), user_agent],
                vec!["Content-Type".into(), "application/x-www-form-urlencoded".into()],
            ]),
            body: Some(post_body),
            timeout: Some(15000),
            proxy: self.proxy.clone(),
            browser: None, ja3: self.override_ja3.clone(), http2fp: self.override_http2fp.clone(),
            follow_redirects: Some(true),
            max_redirects: Some(5),
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((reload_req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("Failed to send reload request".into()))?;
        let reload_resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("Reload response channel closed".into()))?;

        // Extract rresp token
        let rresp = {
            let body = &reload_resp.body;
            let marker = "\"rresp\",\"";
            if let Some(start) = body.find(marker) {
                let after = start + marker.len();
                if let Some(end) = body[after..].find('"') {
                    body[after..after + end].to_string()
                } else {
                    return Err(crate::error::AppError::Pipeline("Failed to parse rresp end".into()));
                }
            } else {
                return Err(crate::error::AppError::Pipeline("rresp not found in reload response".into()));
            }
        };

        self.variables.set_user(&settings.output_var, rresp, settings.capture);
        Ok(())
    }

    // ── XACF Sensor ──

    fn execute_xacf_sensor(&mut self, settings: &XacfSensorSettings) -> crate::error::Result<()> {
        let bundle_id = self.variables.interpolate(&settings.bundle_id);
        let version = self.variables.interpolate(&settings.version);
        let sensor = generate_xacf_sensor_data(&bundle_id, &version);
        self.variables.set_user(&settings.output_var, sensor, settings.capture);
        Ok(())
    }

    // ── Random Data ──

    fn execute_random_data(&mut self, settings: &RandomDataSettings) -> crate::error::Result<()> {
        let value = match settings.data_type {
            RandomDataType::String => random_data::random_string(settings.string_length as usize, &settings.string_charset, &settings.custom_chars),
            RandomDataType::Uuid => random_data::random_uuid(),
            RandomDataType::Number => random_data::random_number(settings.number_min, settings.number_max, settings.number_decimal),
            RandomDataType::Email => random_data::random_email(),
            RandomDataType::FirstName => random_data::random_first_name(),
            RandomDataType::LastName => random_data::random_last_name(),
            RandomDataType::FullName => random_data::random_full_name(),
            RandomDataType::StreetAddress => random_data::random_street_address(),
            RandomDataType::City => random_data::random_city(),
            RandomDataType::State => random_data::random_state(),
            RandomDataType::ZipCode => random_data::random_zip(),
            RandomDataType::PhoneNumber => random_data::random_phone(),
            RandomDataType::Date => random_data::random_date(&settings.date_format, &settings.date_min, &settings.date_max),
        };
        self.variables.set_user(&settings.output_var, value, settings.capture);
        Ok(())
    }

    // ── DataDome Sensor ──

    fn execute_datadome_sensor(&mut self, settings: &DataDomeSensorSettings) -> crate::error::Result<()> {
        let site_url = self.variables.interpolate(&settings.site_url);
        let cookie = self.variables.interpolate(&settings.cookie_datadome);
        let ua = self.variables.interpolate(&settings.user_agent);
        let custom_wasm = if settings.custom_wasm_b64.is_empty() {
            None
        } else {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.decode(settings.custom_wasm_b64.as_bytes()).ok()
        };
        let sensor = datadome::generate_datadome_sensor(
            &site_url,
            &cookie,
            &ua,
            custom_wasm.as_deref(),
        )?;
        self.variables.set_user(&settings.output_var, sensor, settings.capture);
        Ok(())
    }

    // ── Plugin Block ──

    fn execute_plugin_block(&mut self, settings: &PluginBlockSettings) -> crate::error::Result<()> {
        let pm = self.plugin_manager.as_ref()
            .ok_or_else(|| crate::error::AppError::Pipeline("No plugin manager available".into()))?;
        let settings_json = self.variables.interpolate(&settings.settings_json);
        let vars_snapshot = serde_json::to_string(&self.variables.snapshot()).unwrap_or_default();
        let (_, updated_vars, log) = pm.execute_block(&settings.plugin_block_type, &settings_json, &vars_snapshot)
            .map_err(|e| crate::error::AppError::Pipeline(e))?;
        for (k, v) in updated_vars {
            self.variables.set_user(&k, v, settings.capture);
        }
        // If plugin returned a log message and output_var has no value yet, store it
        if !log.is_empty() && self.variables.get(&settings.output_var).unwrap_or_default().is_empty() {
            self.variables.set_user(&settings.output_var, log, settings.capture);
        }
        Ok(())
    }

    // ── TCP Request ──

    async fn execute_tcp_request(&mut self, _block: &Block, settings: &TcpRequestSettings) -> crate::error::Result<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let host = self.variables.interpolate(&settings.host);
        let data = self.variables.interpolate(&settings.data);
        let timeout = std::time::Duration::from_millis(settings.timeout_ms);
        let addr = format!("{}:{}", host, settings.port);

        let stream = tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await
            .map_err(|_| crate::error::AppError::Pipeline(format!("TCP connect timeout: {}", addr)))?
            .map_err(|e| crate::error::AppError::Pipeline(format!("TCP connect failed {}: {}", addr, e)))?;

        let response_body = if settings.use_tls {
            let connector = native_tls::TlsConnector::new()
                .map_err(|e| crate::error::AppError::Pipeline(format!("TLS init error: {}", e)))?;
            let connector = tokio_native_tls::TlsConnector::from(connector);
            let mut tls_stream = connector.connect(&host, stream).await
                .map_err(|e| crate::error::AppError::Pipeline(format!("TLS handshake failed: {}", e)))?;

            if !data.is_empty() {
                tls_stream.write_all(data.as_bytes()).await
                    .map_err(|e| crate::error::AppError::Pipeline(format!("TLS write error: {}", e)))?;
                tls_stream.flush().await.ok();
            }

            let mut buf = vec![0u8; 65536];
            match tokio::time::timeout(timeout, tls_stream.read(&mut buf)).await {
                Ok(Ok(n)) => String::from_utf8_lossy(&buf[..n]).to_string(),
                _ => String::new(),
            }
        } else {
            let mut stream = stream;
            if !data.is_empty() {
                stream.write_all(data.as_bytes()).await
                    .map_err(|e| crate::error::AppError::Pipeline(format!("TCP write error: {}", e)))?;
                stream.flush().await.ok();
            }

            let mut buf = vec![0u8; 65536];
            match tokio::time::timeout(timeout, stream.read(&mut buf)).await {
                Ok(Ok(n)) => String::from_utf8_lossy(&buf[..n]).to_string(),
                _ => String::new(),
            }
        };

        self.variables.set_user(&settings.output_var, response_body.clone(), settings.capture);

        if let Some(last) = self.block_results.last_mut() {
            last.request = Some(RequestInfo {
                method: if settings.use_tls { "TCP+TLS".into() } else { "TCP".into() },
                url: addr,
                headers: vec![],
                body: data,
            });
            last.response = Some(ResponseInfo {
                status_code: if response_body.is_empty() { 0 } else { 200 },
                headers: std::collections::HashMap::new(),
                body: response_body,
                final_url: String::new(),
                cookies: std::collections::HashMap::new(),
                timing_ms: 0,
            });
        }

        Ok(())
    }

    // ── UDP Request ──

    async fn execute_udp_request(&mut self, _block: &Block, settings: &UdpRequestSettings) -> crate::error::Result<()> {
        let host = self.variables.interpolate(&settings.host);
        let data = self.variables.interpolate(&settings.data);
        let timeout = std::time::Duration::from_millis(settings.timeout_ms);
        let addr = format!("{}:{}", host, settings.port);

        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await
            .map_err(|e| crate::error::AppError::Pipeline(format!("UDP bind error: {}", e)))?;

        socket.send_to(data.as_bytes(), &addr).await
            .map_err(|e| crate::error::AppError::Pipeline(format!("UDP send error: {}", e)))?;

        let mut buf = vec![0u8; 65536];
        let response_body = match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
            Ok(Ok((n, _src))) => String::from_utf8_lossy(&buf[..n]).to_string(),
            _ => String::new(),
        };

        self.variables.set_user(&settings.output_var, response_body.clone(), settings.capture);

        if let Some(last) = self.block_results.last_mut() {
            last.request = Some(RequestInfo {
                method: "UDP".into(),
                url: addr,
                headers: vec![],
                body: data,
            });
            last.response = Some(ResponseInfo {
                status_code: if response_body.is_empty() { 0 } else { 200 },
                headers: std::collections::HashMap::new(),
                body: response_body,
                final_url: String::new(),
                cookies: std::collections::HashMap::new(),
                timing_ms: 0,
            });
        }

        Ok(())
    }

    // ── FTP Request ──

    async fn execute_ftp_request(&mut self, _block: &Block, settings: &FtpRequestSettings) -> crate::error::Result<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

        let host = self.variables.interpolate(&settings.host);
        let username = self.variables.interpolate(&settings.username);
        let password = self.variables.interpolate(&settings.password);
        let command = self.variables.interpolate(&settings.command);
        let timeout = std::time::Duration::from_millis(settings.timeout_ms);
        let addr = format!("{}:{}", host, settings.port);

        let stream = tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await
            .map_err(|_| crate::error::AppError::Pipeline(format!("FTP connect timeout: {}", addr)))?
            .map_err(|e| crate::error::AppError::Pipeline(format!("FTP connect failed {}: {}", addr, e)))?;

        let (reader, writer) = tokio::io::split(stream);
        let mut reader = tokio::io::BufReader::new(reader);
        let mut writer = writer;
        let mut transcript = String::new();

        // Read banner
        let mut line = String::new();
        if let Ok(Ok(n)) = tokio::time::timeout(timeout, reader.read_line(&mut line)).await {
            if n > 0 { transcript.push_str(&format!("S: {}", line)); }
        }

        let commands = vec![
            format!("USER {}", username),
            format!("PASS {}", password),
            command.clone(),
            "QUIT".into(),
        ];

        let mut last_code: u16 = 0;
        for cmd in &commands {
            if cmd.is_empty() { continue; }
            writer.write_all(format!("{}\r\n", cmd).as_bytes()).await.ok();
            writer.flush().await.ok();
            transcript.push_str(&format!("C: {}\r\n", cmd));

            // Read FTP response (multi-line: "123-...\r\n123 ...\r\n")
            loop {
                line.clear();
                match tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut line)).await {
                    Ok(Ok(0)) | Err(_) | Ok(Err(_)) => break,
                    Ok(Ok(_)) => {
                        transcript.push_str(&format!("S: {}", line));
                        if let Ok(code) = line.get(..3).unwrap_or("").parse::<u16>() {
                            last_code = code;
                        }
                        if line.len() >= 4 && line.as_bytes()[3] != b'-' { break; }
                    }
                }
            }
        }

        self.variables.set_user(&settings.output_var, transcript.clone(), settings.capture);

        if let Some(last) = self.block_results.last_mut() {
            last.request = Some(RequestInfo {
                method: "FTP".into(),
                url: addr,
                headers: vec![],
                body: commands.iter().filter(|c| !c.is_empty()).cloned().collect::<Vec<_>>().join("\r\n"),
            });
            last.response = Some(ResponseInfo {
                status_code: last_code,
                headers: std::collections::HashMap::new(),
                body: transcript,
                final_url: String::new(),
                cookies: std::collections::HashMap::new(),
                timing_ms: 0,
            });
        }

        Ok(())
    }

    // ── SSH Request (banner grab + auth attempt via raw protocol) ──

    async fn execute_ssh_request(&mut self, _block: &Block, settings: &SshRequestSettings) -> crate::error::Result<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let host = self.variables.interpolate(&settings.host);
        let timeout = std::time::Duration::from_millis(settings.timeout_ms);
        let addr = format!("{}:{}", host, settings.port);

        let mut stream = tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await
            .map_err(|_| crate::error::AppError::Pipeline(format!("SSH connect timeout: {}", addr)))?
            .map_err(|e| crate::error::AppError::Pipeline(format!("SSH connect failed {}: {}", addr, e)))?;

        // Read server banner (e.g., "SSH-2.0-OpenSSH_8.9p1\r\n")
        let mut buf = vec![0u8; 4096];
        let banner = match tokio::time::timeout(timeout, stream.read(&mut buf)).await {
            Ok(Ok(n)) => String::from_utf8_lossy(&buf[..n]).to_string(),
            _ => String::new(),
        };

        // Send client banner
        stream.write_all(b"SSH-2.0-ReqFlow_1.0\r\n").await.ok();

        let transcript = format!("S: {}\nC: SSH-2.0-ReqFlow_1.0\n\nNote: Full SSH auth requires ssh2 crate. Banner exchange completed.", banner.trim());
        self.variables.set_user(&settings.output_var, transcript.clone(), settings.capture);

        if let Some(last) = self.block_results.last_mut() {
            last.request = Some(RequestInfo {
                method: "SSH".into(),
                url: addr,
                headers: vec![],
                body: format!("user={} command={}", settings.username, settings.command),
            });
            last.response = Some(ResponseInfo {
                status_code: if banner.contains("SSH-2.0") { 200 } else { 0 },
                headers: {
                    let mut h = std::collections::HashMap::new();
                    h.insert("Server-Banner".into(), banner.trim().to_string());
                    h
                },
                body: transcript,
                final_url: String::new(),
                cookies: std::collections::HashMap::new(),
                timing_ms: 0,
            });
        }

        Ok(())
    }

    // ── IMAP Request ──

    async fn execute_imap_request(&mut self, _block: &Block, settings: &ImapRequestSettings) -> crate::error::Result<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

        let host = self.variables.interpolate(&settings.host);
        let username = self.variables.interpolate(&settings.username);
        let password = self.variables.interpolate(&settings.password);
        let command = self.variables.interpolate(&settings.command);
        let timeout = std::time::Duration::from_millis(settings.timeout_ms);
        let addr = format!("{}:{}", host, settings.port);

        let stream = tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await
            .map_err(|_| crate::error::AppError::Pipeline(format!("IMAP connect timeout: {}", addr)))?
            .map_err(|e| crate::error::AppError::Pipeline(format!("IMAP connect failed {}: {}", addr, e)))?;

        let mut transcript = String::new();
        let mut last_ok = false;

        if settings.use_tls {
            let connector = native_tls::TlsConnector::new()
                .map_err(|e| crate::error::AppError::Pipeline(format!("TLS error: {}", e)))?;
            let connector = tokio_native_tls::TlsConnector::from(connector);
            let tls = connector.connect(&host, stream).await
                .map_err(|e| crate::error::AppError::Pipeline(format!("TLS handshake: {}", e)))?;
            let (reader, writer) = tokio::io::split(tls);
            let mut reader = tokio::io::BufReader::new(reader);
            let mut writer = writer;

            // Banner
            let mut line = String::new();
            if let Ok(Ok(n)) = tokio::time::timeout(timeout, reader.read_line(&mut line)).await {
                if n > 0 { transcript.push_str(&format!("S: {}", line)); }
            }

            // LOGIN
            let login_cmd = format!("a001 LOGIN {} {}", username, password);
            writer.write_all(format!("{}\r\n", login_cmd).as_bytes()).await.ok();
            writer.flush().await.ok();
            transcript.push_str(&format!("C: a001 LOGIN {} ****\r\n", username));
            loop {
                line.clear();
                match tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut line)).await {
                    Ok(Ok(0)) | Err(_) | Ok(Err(_)) => break,
                    Ok(Ok(_)) => {
                        transcript.push_str(&format!("S: {}", line));
                        if line.starts_with("a001 ") { last_ok = line.contains(" OK "); break; }
                    }
                }
            }

            // Extra command if provided and login succeeded
            if last_ok && !command.is_empty() && command != "LOGIN" {
                let extra = format!("a002 {}", command);
                writer.write_all(format!("{}\r\n", extra).as_bytes()).await.ok();
                writer.flush().await.ok();
                transcript.push_str(&format!("C: {}\r\n", extra));
                loop {
                    line.clear();
                    match tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut line)).await {
                        Ok(Ok(0)) | Err(_) | Ok(Err(_)) => break,
                        Ok(Ok(_)) => {
                            transcript.push_str(&format!("S: {}", line));
                            if line.starts_with("a002 ") { break; }
                        }
                    }
                }
            }

            // LOGOUT
            writer.write_all(b"a003 LOGOUT\r\n").await.ok();
            writer.flush().await.ok();
        } else {
            let (reader, writer) = tokio::io::split(stream);
            let mut reader = tokio::io::BufReader::new(reader);
            let mut writer = writer;

            let mut line = String::new();
            if let Ok(Ok(n)) = tokio::time::timeout(timeout, reader.read_line(&mut line)).await {
                if n > 0 { transcript.push_str(&format!("S: {}", line)); }
            }

            let login_cmd = format!("a001 LOGIN {} {}", username, password);
            writer.write_all(format!("{}\r\n", login_cmd).as_bytes()).await.ok();
            writer.flush().await.ok();
            transcript.push_str(&format!("C: a001 LOGIN {} ****\r\n", username));
            loop {
                line.clear();
                match tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut line)).await {
                    Ok(Ok(0)) | Err(_) | Ok(Err(_)) => break,
                    Ok(Ok(_)) => {
                        transcript.push_str(&format!("S: {}", line));
                        if line.starts_with("a001 ") { last_ok = line.contains(" OK "); break; }
                    }
                }
            }

            if last_ok && !command.is_empty() && command != "LOGIN" {
                let extra = format!("a002 {}", command);
                writer.write_all(format!("{}\r\n", extra).as_bytes()).await.ok();
                writer.flush().await.ok();
                transcript.push_str(&format!("C: {}\r\n", extra));
                loop {
                    line.clear();
                    match tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut line)).await {
                        Ok(Ok(0)) | Err(_) | Ok(Err(_)) => break,
                        Ok(Ok(_)) => {
                            transcript.push_str(&format!("S: {}", line));
                            if line.starts_with("a002 ") { break; }
                        }
                    }
                }
            }

            writer.write_all(b"a003 LOGOUT\r\n").await.ok();
            writer.flush().await.ok();
        }

        self.variables.set_user(&settings.output_var, transcript.clone(), settings.capture);

        if let Some(last) = self.block_results.last_mut() {
            last.request = Some(RequestInfo {
                method: "IMAP".into(),
                url: addr,
                headers: vec![],
                body: format!("LOGIN {} ****\n{}", username, command),
            });
            last.response = Some(ResponseInfo {
                status_code: if last_ok { 200 } else { 401 },
                headers: std::collections::HashMap::new(),
                body: transcript,
                final_url: String::new(),
                cookies: std::collections::HashMap::new(),
                timing_ms: 0,
            });
        }

        Ok(())
    }

    // ── SMTP Request ──

    async fn execute_smtp_request(&mut self, _block: &Block, settings: &SmtpRequestSettings) -> crate::error::Result<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

        let host = self.variables.interpolate(&settings.host);
        let username = self.variables.interpolate(&settings.username);
        let password = self.variables.interpolate(&settings.password);
        let command = self.variables.interpolate(&settings.command);
        let timeout = std::time::Duration::from_millis(settings.timeout_ms);
        let addr = format!("{}:{}", host, settings.port);

        let stream = tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await
            .map_err(|_| crate::error::AppError::Pipeline(format!("SMTP connect timeout: {}", addr)))?
            .map_err(|e| crate::error::AppError::Pipeline(format!("SMTP connect failed {}: {}", addr, e)))?;

        let mut transcript = String::new();
        let mut auth_ok = false;

        // Helper closure for reading SMTP multi-line responses
        macro_rules! smtp_read {
            ($reader:expr, $transcript:expr) => {{
                let mut last_code: u16 = 0;
                loop {
                    let mut line = String::new();
                    match tokio::time::timeout(std::time::Duration::from_secs(5), $reader.read_line(&mut line)).await {
                        Ok(Ok(0)) | Err(_) | Ok(Err(_)) => break,
                        Ok(Ok(_)) => {
                            $transcript.push_str(&format!("S: {}", line));
                            if let Ok(c) = line.get(..3).unwrap_or("").parse::<u16>() { last_code = c; }
                            if line.len() >= 4 && line.as_bytes()[3] != b'-' { break; }
                        }
                    }
                }
                last_code
            }};
        }

        if settings.use_tls {
            let connector = native_tls::TlsConnector::new()
                .map_err(|e| crate::error::AppError::Pipeline(format!("TLS error: {}", e)))?;
            let connector = tokio_native_tls::TlsConnector::from(connector);
            let tls = connector.connect(&host, stream).await
                .map_err(|e| crate::error::AppError::Pipeline(format!("TLS handshake: {}", e)))?;
            let (reader, writer) = tokio::io::split(tls);
            let mut reader = tokio::io::BufReader::new(reader);
            let mut writer = writer;

            smtp_read!(reader, transcript);

            // EHLO
            writer.write_all(format!("EHLO reqflow\r\n").as_bytes()).await.ok();
            writer.flush().await.ok();
            transcript.push_str("C: EHLO reqflow\r\n");
            smtp_read!(reader, transcript);

            // AUTH LOGIN
            if !username.is_empty() {
                use base64::Engine;
                writer.write_all(b"AUTH LOGIN\r\n").await.ok();
                writer.flush().await.ok();
                transcript.push_str("C: AUTH LOGIN\r\n");
                smtp_read!(reader, transcript);

                let b64_user = base64::engine::general_purpose::STANDARD.encode(username.as_bytes());
                writer.write_all(format!("{}\r\n", b64_user).as_bytes()).await.ok();
                writer.flush().await.ok();
                transcript.push_str(&format!("C: {}\r\n", b64_user));
                smtp_read!(reader, transcript);

                let b64_pass = base64::engine::general_purpose::STANDARD.encode(password.as_bytes());
                writer.write_all(format!("{}\r\n", b64_pass).as_bytes()).await.ok();
                writer.flush().await.ok();
                transcript.push_str("C: ****\r\n");
                let code = smtp_read!(reader, transcript);
                auth_ok = code == 235;
            }

            // Extra command
            if !command.is_empty() && command != "EHLO" {
                writer.write_all(format!("{}\r\n", command).as_bytes()).await.ok();
                writer.flush().await.ok();
                transcript.push_str(&format!("C: {}\r\n", command));
                smtp_read!(reader, transcript);
            }

            writer.write_all(b"QUIT\r\n").await.ok();
            writer.flush().await.ok();
        } else {
            let (reader, writer) = tokio::io::split(stream);
            let mut reader = tokio::io::BufReader::new(reader);
            let mut writer = writer;

            smtp_read!(reader, transcript);

            writer.write_all(b"EHLO reqflow\r\n").await.ok();
            writer.flush().await.ok();
            transcript.push_str("C: EHLO reqflow\r\n");
            smtp_read!(reader, transcript);

            if !username.is_empty() {
                use base64::Engine;
                writer.write_all(b"AUTH LOGIN\r\n").await.ok();
                writer.flush().await.ok();
                transcript.push_str("C: AUTH LOGIN\r\n");
                smtp_read!(reader, transcript);

                let b64_user = base64::engine::general_purpose::STANDARD.encode(username.as_bytes());
                writer.write_all(format!("{}\r\n", b64_user).as_bytes()).await.ok();
                writer.flush().await.ok();
                transcript.push_str(&format!("C: {}\r\n", b64_user));
                smtp_read!(reader, transcript);

                let b64_pass = base64::engine::general_purpose::STANDARD.encode(password.as_bytes());
                writer.write_all(format!("{}\r\n", b64_pass).as_bytes()).await.ok();
                writer.flush().await.ok();
                transcript.push_str("C: ****\r\n");
                let code = smtp_read!(reader, transcript);
                auth_ok = code == 235;
            }

            if !command.is_empty() && command != "EHLO" {
                writer.write_all(format!("{}\r\n", command).as_bytes()).await.ok();
                writer.flush().await.ok();
                transcript.push_str(&format!("C: {}\r\n", command));
                smtp_read!(reader, transcript);
            }

            writer.write_all(b"QUIT\r\n").await.ok();
            writer.flush().await.ok();
        }

        self.variables.set_user(&settings.output_var, transcript.clone(), settings.capture);

        if let Some(last) = self.block_results.last_mut() {
            last.request = Some(RequestInfo {
                method: "SMTP".into(),
                url: addr,
                headers: vec![],
                body: format!("AUTH {} ****", username),
            });
            last.response = Some(ResponseInfo {
                status_code: if auth_ok { 235 } else { 535 },
                headers: std::collections::HashMap::new(),
                body: transcript,
                final_url: String::new(),
                cookies: std::collections::HashMap::new(),
                timing_ms: 0,
            });
        }

        Ok(())
    }

    // ── POP3 Request ──

    async fn execute_pop_request(&mut self, _block: &Block, settings: &PopRequestSettings) -> crate::error::Result<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

        let host = self.variables.interpolate(&settings.host);
        let username = self.variables.interpolate(&settings.username);
        let password = self.variables.interpolate(&settings.password);
        let command = self.variables.interpolate(&settings.command);
        let timeout = std::time::Duration::from_millis(settings.timeout_ms);
        let addr = format!("{}:{}", host, settings.port);

        let stream = tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await
            .map_err(|_| crate::error::AppError::Pipeline(format!("POP3 connect timeout: {}", addr)))?
            .map_err(|e| crate::error::AppError::Pipeline(format!("POP3 connect failed {}: {}", addr, e)))?;

        let mut transcript = String::new();
        #[allow(unused_assignments)]
        let mut auth_ok = false;

        macro_rules! pop_read_line {
            ($reader:expr, $transcript:expr) => {{
                let mut line = String::new();
                match tokio::time::timeout(std::time::Duration::from_secs(5), $reader.read_line(&mut line)).await {
                    Ok(Ok(n)) if n > 0 => {
                        $transcript.push_str(&format!("S: {}", line));
                        line.starts_with("+OK")
                    }
                    _ => false,
                }
            }};
        }

        if settings.use_tls {
            let connector = native_tls::TlsConnector::new()
                .map_err(|e| crate::error::AppError::Pipeline(format!("TLS error: {}", e)))?;
            let connector = tokio_native_tls::TlsConnector::from(connector);
            let tls = connector.connect(&host, stream).await
                .map_err(|e| crate::error::AppError::Pipeline(format!("TLS handshake: {}", e)))?;
            let (reader, writer) = tokio::io::split(tls);
            let mut reader = tokio::io::BufReader::new(reader);
            let mut writer = writer;

            pop_read_line!(reader, transcript); // Banner

            writer.write_all(format!("USER {}\r\n", username).as_bytes()).await.ok();
            writer.flush().await.ok();
            transcript.push_str(&format!("C: USER {}\r\n", username));
            pop_read_line!(reader, transcript);

            writer.write_all(format!("PASS {}\r\n", password).as_bytes()).await.ok();
            writer.flush().await.ok();
            transcript.push_str("C: PASS ****\r\n");
            auth_ok = pop_read_line!(reader, transcript);

            if auth_ok && !command.is_empty() && command != "STAT" || command == "STAT" {
                writer.write_all(format!("{}\r\n", command).as_bytes()).await.ok();
                writer.flush().await.ok();
                transcript.push_str(&format!("C: {}\r\n", command));
                pop_read_line!(reader, transcript);
            }

            writer.write_all(b"QUIT\r\n").await.ok();
            writer.flush().await.ok();
        } else {
            let (reader, writer) = tokio::io::split(stream);
            let mut reader = tokio::io::BufReader::new(reader);
            let mut writer = writer;

            pop_read_line!(reader, transcript);

            writer.write_all(format!("USER {}\r\n", username).as_bytes()).await.ok();
            writer.flush().await.ok();
            transcript.push_str(&format!("C: USER {}\r\n", username));
            pop_read_line!(reader, transcript);

            writer.write_all(format!("PASS {}\r\n", password).as_bytes()).await.ok();
            writer.flush().await.ok();
            transcript.push_str("C: PASS ****\r\n");
            auth_ok = pop_read_line!(reader, transcript);

            if auth_ok && !command.is_empty() {
                writer.write_all(format!("{}\r\n", command).as_bytes()).await.ok();
                writer.flush().await.ok();
                transcript.push_str(&format!("C: {}\r\n", command));
                pop_read_line!(reader, transcript);
            }

            writer.write_all(b"QUIT\r\n").await.ok();
            writer.flush().await.ok();
        }

        self.variables.set_user(&settings.output_var, transcript.clone(), settings.capture);

        if let Some(last) = self.block_results.last_mut() {
            last.request = Some(RequestInfo {
                method: "POP3".into(),
                url: addr,
                headers: vec![],
                body: format!("USER {} PASS ****", username),
            });
            last.response = Some(ResponseInfo {
                status_code: if auth_ok { 200 } else { 401 },
                headers: std::collections::HashMap::new(),
                body: transcript,
                final_url: String::new(),
                cookies: std::collections::HashMap::new(),
                timing_ms: 0,
            });
        }

        Ok(())
    }

    // ── Captcha Solver (CapSolver / 2Captcha API) ──

    async fn execute_captcha_solver(
        &mut self,
        settings: &CaptchaSolverSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        let api_key = self.variables.interpolate(&settings.api_key);
        let site_key = self.variables.interpolate(&settings.site_key);
        let page_url = self.variables.interpolate(&settings.page_url);
        let timeout = std::time::Duration::from_millis(settings.timeout_ms);

        let (create_url, create_body): (String, String) = match settings.solver_service.as_str() {
            "capsolver" => {
                ("https://api.capsolver.com/createTask".into(),
                 serde_json::json!({
                     "clientKey": api_key,
                     "task": {
                         "type": match settings.captcha_type.as_str() {
                             "RecaptchaV3" => "RecaptchaV3TaskProxyless",
                             "HCaptcha" => "HCaptchaTaskProxyless",
                             _ => "RecaptchaV2TaskProxyless",
                         },
                         "websiteURL": page_url,
                         "websiteKey": site_key,
                     }
                 }).to_string())
            }
            _ => { // 2captcha
                ("https://api.2captcha.com/createTask".into(),
                 serde_json::json!({
                     "clientKey": api_key,
                     "task": {
                         "type": match settings.captcha_type.as_str() {
                             "RecaptchaV3" => "RecaptchaV3TaskProxyless",
                             "HCaptcha" => "HCaptchaTaskProxyless",
                             _ => "NoCaptchaTaskProxyless",
                         },
                         "websiteURL": page_url,
                         "websiteKey": site_key,
                     }
                 }).to_string())
            }
        };

        // Create task
        let req = SidecarRequest {
            id: Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some("POST".into()),
            url: Some(create_url.clone()),
            headers: Some(vec![vec!["Content-Type".into(), "application/json".into()]]),
            body: Some(create_body),
            timeout: Some(30000),
            proxy: None, browser: None, ja3: None, http2fp: None,
            follow_redirects: Some(true), max_redirects: Some(5),
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("Failed to send captcha create".into()))?;
        let resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("Captcha create channel closed".into()))?;

        let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap_or_default();
        let task_id = json["taskId"].as_str().unwrap_or("").to_string();
        if task_id.is_empty() {
            return Err(crate::error::AppError::Pipeline(format!("Captcha task creation failed: {}", resp.body)));
        }

        // Poll for result
        let poll_url = match settings.solver_service.as_str() {
            "capsolver" => "https://api.capsolver.com/getTaskResult",
            _ => "https://api.2captcha.com/getTaskResult",
        };
        let start = std::time::Instant::now();
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            let poll_body = serde_json::json!({
                "clientKey": api_key,
                "taskId": task_id,
            }).to_string();

            let req = SidecarRequest {
                id: Uuid::new_v4().to_string(),
                action: "request".into(),
                session: self.session_id.clone(),
                method: Some("POST".into()),
                url: Some(poll_url.into()),
                headers: Some(vec![vec!["Content-Type".into(), "application/json".into()]]),
                body: Some(poll_body),
                timeout: Some(15000),
                proxy: None, browser: None, ja3: None, http2fp: None,
                follow_redirects: Some(true), max_redirects: Some(5),
            };

            let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
            sidecar_tx.send((req, resp_tx)).await.ok();
            if let Ok(resp) = resp_rx.await {
                let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap_or_default();
                let status = json["status"].as_str().unwrap_or("");
                if status == "ready" {
                    let token = json["solution"]["gRecaptchaResponse"]
                        .as_str()
                        .or_else(|| json["solution"]["token"].as_str())
                        .unwrap_or("")
                        .to_string();
                    self.variables.set_user(&settings.output_var, token, settings.capture);
                    return Ok(());
                }
            }

            if start.elapsed() > timeout {
                return Err(crate::error::AppError::Pipeline("Captcha solver timeout".into()));
            }
        }
    }

    // ── Cloudflare Bypass (via FlareSolverr) ──

    async fn execute_cloudflare_bypass(
        &mut self,
        settings: &CloudflareBypassSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        let url = self.variables.interpolate(&settings.url);
        let flaresolverr = self.variables.interpolate(&settings.flaresolverr_url);

        let body = serde_json::json!({
            "cmd": "request.get",
            "url": url,
            "maxTimeout": settings.max_timeout_ms,
        }).to_string();

        let req = SidecarRequest {
            id: Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some("POST".into()),
            url: Some(flaresolverr),
            headers: Some(vec![vec!["Content-Type".into(), "application/json".into()]]),
            body: Some(body),
            timeout: Some(settings.max_timeout_ms as i64),
            proxy: None, browser: None, ja3: None, http2fp: None,
            follow_redirects: Some(true), max_redirects: Some(5),
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("FlareSolverr send failed".into()))?;
        let resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("FlareSolverr channel closed".into()))?;

        let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap_or_default();
        let cookies = json["solution"]["cookies"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| {
                        let name = c["name"].as_str()?;
                        let value = c["value"].as_str()?;
                        Some(format!("{}={}", name, value))
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            .unwrap_or_default();

        let user_agent = json["solution"]["userAgent"].as_str().unwrap_or("").to_string();
        if !user_agent.is_empty() {
            self.variables.set_user("CF_USERAGENT", user_agent, false);
        }

        self.variables.set_user(&settings.output_var, cookies, settings.capture);
        Ok(())
    }

    // ── Laravel CSRF (HTTP GET + parse token) ──

    async fn execute_laravel_csrf(
        &mut self,
        settings: &LaravelCsrfSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        let url = self.variables.interpolate(&settings.url);
        let selector = self.variables.interpolate(&settings.csrf_selector);

        let req = SidecarRequest {
            id: Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some("GET".into()),
            url: Some(url),
            headers: Some(vec![]),
            body: Some(String::new()),
            timeout: Some(settings.timeout_ms as i64),
            proxy: self.proxy.clone(),
            browser: None,
            ja3: self.override_ja3.clone(),
            http2fp: self.override_http2fp.clone(),
            follow_redirects: Some(true),
            max_redirects: Some(5),
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("CSRF request send failed".into()))?;
        let resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("CSRF response channel closed".into()))?;

        // Parse CSRF token from HTML using CSS selector
        let doc = scraper::Html::parse_document(&resp.body);
        let sel = scraper::Selector::parse(&selector)
            .map_err(|e| crate::error::AppError::Pipeline(format!("Invalid CSRF selector: {:?}", e)))?;

        let token = doc.select(&sel)
            .next()
            .and_then(|el| el.value().attr("value").or_else(|| el.value().attr("content")))
            .unwrap_or("")
            .to_string();

        // Also extract XSRF cookie if present
        if !settings.cookie_name.is_empty() {
            if let Some(ref cookies) = resp.cookies {
                if let Some(cookie_val) = cookies.get(&settings.cookie_name) {
                    self.variables.set_user(&format!("{}_COOKIE", settings.output_var), cookie_val.clone(), false);
                }
            }
        }

        self.variables.set_user(&settings.output_var, token, settings.capture);
        Ok(())
    }

    // ── Execution log summary ──

    fn block_execution_log(&self, block: &Block) -> String {
        match &block.settings {
            BlockSettings::HttpRequest(_) => {
                if let Some(br) = self.block_results.last() {
                    let method = br.request.as_ref().map(|r| r.method.as_str()).unwrap_or("?");
                    let url = br.request.as_ref().map(|r| truncate_display(&r.url, 60)).unwrap_or_else(|| "?".into());
                    let status = br.response.as_ref().map(|r| r.status_code).unwrap_or(0);
                    let timing = br.response.as_ref().map(|r| r.timing_ms).unwrap_or(0);
                    format!("{} {} → {} ({}ms)", method, url, status, timing)
                } else {
                    "HTTP request completed".into()
                }
            }
            BlockSettings::ParseLR(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} = {}", s.output_var, truncate_display(&val, 60))
            }
            BlockSettings::ParseRegex(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} = {}", s.output_var, truncate_display(&val, 60))
            }
            BlockSettings::ParseJSON(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} = {}", s.output_var, truncate_display(&val, 60))
            }
            BlockSettings::ParseCSS(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} = {}", s.output_var, truncate_display(&val, 60))
            }
            BlockSettings::ParseXPath(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} = {}", s.output_var, truncate_display(&val, 60))
            }
            BlockSettings::ParseCookie(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} = {}", s.output_var, truncate_display(&val, 60))
            }
            BlockSettings::KeyCheck(_) => {
                format!("status → {:?}", self.status)
            }
            BlockSettings::StringFunction(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{:?}({}) → {} = {}", s.function_type, s.input_var, s.output_var, truncate_display(&val, 40))
            }
            BlockSettings::CryptoFunction(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{:?}({}) → {} = {}", s.function_type, s.input_var, s.output_var, truncate_display(&val, 40))
            }
            BlockSettings::ListFunction(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{:?}({}) → {} = {}", s.function_type, s.input_var, s.output_var, truncate_display(&val, 40))
            }
            BlockSettings::ConversionFunction(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{}→{}({}) = {}", s.from_type, s.to_type, s.input_var, truncate_display(&val, 40))
            }
            BlockSettings::DateFunction(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{:?} → {} = {}", s.function_type, s.output_var, truncate_display(&val, 40))
            }
            BlockSettings::CaseSwitch(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("switch {} → {} = {}", s.input_var, s.output_var, val)
            }
            BlockSettings::CookieContainer(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                let count = val.matches(';').count() + if val.is_empty() { 0 } else { 1 };
                format!("{} cookies → {}", count, s.output_var)
            }
            BlockSettings::IfElse(_) => "condition evaluated, branch executed".into(),
            BlockSettings::Loop(s) => {
                match s.loop_type {
                    LoopType::ForEach => format!("foreach {} ({} blocks)", s.item_var, s.blocks.len()),
                    LoopType::Repeat => format!("repeat {}x ({} blocks)", s.count, s.blocks.len()),
                }
            }
            BlockSettings::Delay(s) => {
                if s.min_ms == s.max_ms { format!("{}ms", s.min_ms) }
                else { format!("{}-{}ms", s.min_ms, s.max_ms) }
            }
            BlockSettings::SetVariable(s) => {
                let val = self.variables.get(&s.name).unwrap_or_default();
                format!("{} = {}", s.name, truncate_display(&val, 60))
            }
            BlockSettings::Log(_) => String::new(), // Already logged by handler
            BlockSettings::ClearCookies => "session cookies cleared".into(),
            BlockSettings::Script(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} = {}", s.output_var, truncate_display(&val, 60))
            }
            // Browser
            BlockSettings::BrowserOpen(s) => {
                format!("launched {} {}", s.browser_type, if s.headless { "(headless)" } else { "(headed)" })
            }
            BlockSettings::NavigateTo(s) => format!("navigated to {}", truncate_display(&s.url, 60)),
            BlockSettings::ClickElement(s) => format!("clicked {}", s.selector),
            BlockSettings::TypeText(s) => format!("typed into {}", s.selector),
            BlockSettings::WaitForElement(s) => format!("waited for {} [{}]", s.selector, s.state),
            BlockSettings::GetElementText(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} = {}", s.output_var, truncate_display(&val, 60))
            }
            BlockSettings::Screenshot(s) => {
                if s.full_page { "full page screenshot".into() }
                else if !s.selector.is_empty() { format!("screenshot of {}", s.selector) }
                else { "viewport screenshot".into() }
            }
            BlockSettings::ExecuteJs(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} = {}", s.output_var, truncate_display(&val, 60))
            }
            // Networking
            BlockSettings::Webhook(s) => format!("{} {}", s.method, truncate_display(&s.url, 60)),
            BlockSettings::WebSocket(s) => format!("{} {}", s.action, truncate_display(&s.url, 60)),
            // Protocol
            BlockSettings::TcpRequest(s) => format!("{}:{}", s.host, s.port),
            BlockSettings::UdpRequest(s) => format!("{}:{}", s.host, s.port),
            BlockSettings::FtpRequest(s) => format!("{}:{} {}", s.host, s.port, s.command),
            BlockSettings::SshRequest(s) => format!("{}:{}", s.host, s.port),
            BlockSettings::ImapRequest(s) => format!("{}:{} {}", s.host, s.port, s.command),
            BlockSettings::SmtpRequest(s) => format!("{}:{} {}", s.host, s.port, s.command),
            BlockSettings::PopRequest(s) => format!("{}:{} {}", s.host, s.port, s.command),
            // Bypass
            BlockSettings::CaptchaSolver(s) => format!("{} {}", s.solver_service, s.captcha_type),
            BlockSettings::CloudflareBypass(s) => format!("{}", truncate_display(&s.url, 60)),
            BlockSettings::LaravelCsrf(s) => format!("{} → {}", truncate_display(&s.url, 40), s.output_var),
            // New blocks
            BlockSettings::RandomUserAgent(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} = {}", s.output_var, truncate_display(&val, 60))
            }
            BlockSettings::OcrCaptcha(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} = {}", s.output_var, truncate_display(&val, 60))
            }
            BlockSettings::RecaptchaInvisible(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} = {}", s.output_var, truncate_display(&val, 40))
            }
            BlockSettings::XacfSensor(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} → {} ({}b)", s.bundle_id, s.output_var, val.len())
            }
            BlockSettings::RandomData(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{:?} → {} = {}", s.data_type, s.output_var, truncate_display(&val, 60))
            }
            BlockSettings::DataDomeSensor(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                format!("{} → {} ({}b)", truncate_display(&s.site_url, 30), s.output_var, val.len())
            }
            BlockSettings::Plugin(s) => {
                let val = self.variables.get(&s.output_var).unwrap_or_default();
                if val.is_empty() {
                    format!("plugin: {} (no output)", s.plugin_block_type)
                } else {
                    format!("{} = {}", s.output_var, truncate_display(&val, 60))
                }
            }
            BlockSettings::Group(s) => {
                format!("{} block{}", s.blocks.len(), if s.blocks.len() != 1 { "s" } else { "" })
            }
        }
    }
}

fn truncate_display(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max])
    } else {
        s.to_string()
    }
}

fn elapsed_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn urlencoding(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// Built-in user agent strings: (ua_string, browser, platform)
const BUILTIN_USER_AGENTS: &[(&str, &str, &str)] = &[
    ("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36", "Chrome", "Desktop"),
    ("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36", "Chrome", "Desktop"),
    ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36", "Chrome", "Desktop"),
    ("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36", "Chrome", "Desktop"),
    ("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0", "Firefox", "Desktop"),
    ("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:132.0) Gecko/20100101 Firefox/132.0", "Firefox", "Desktop"),
    ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:133.0) Gecko/20100101 Firefox/133.0", "Firefox", "Desktop"),
    ("Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0", "Firefox", "Desktop"),
    ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Safari/605.1.15", "Safari", "Desktop"),
    ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.6 Safari/605.1.15", "Safari", "Desktop"),
    ("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0", "Edge", "Desktop"),
    ("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0", "Edge", "Desktop"),
    // Mobile
    ("Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1", "Safari", "Mobile"),
    ("Mozilla/5.0 (iPhone; CPU iPhone OS 17_7 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.7 Mobile/15E148 Safari/604.1", "Safari", "Mobile"),
    ("Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36", "Chrome", "Mobile"),
    ("Mozilla/5.0 (Linux; Android 14; SM-S921B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36", "Chrome", "Mobile"),
    ("Mozilla/5.0 (Linux; Android 13; SM-A536B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Mobile Safari/537.36", "Chrome", "Mobile"),
    ("Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/131.0.6778.73 Mobile/15E148 Safari/604.1", "Chrome", "Mobile"),
    ("Mozilla/5.0 (Android 14; Mobile; rv:133.0) Gecko/133.0 Firefox/133.0", "Firefox", "Mobile"),
    ("Mozilla/5.0 (iPad; CPU OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1", "Safari", "Tablet"),
    ("Mozilla/5.0 (Linux; Android 14; SM-X710) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36", "Chrome", "Tablet"),
];

/// Generate x-acf-sensor-data payload for Akamai Bot Manager
fn generate_xacf_sensor_data(bundle_id: &str, version: &str) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let device_model = "iPhone14,3";
    let os_version = "18.1";
    let screen_w = 1170;
    let screen_h = 2532;

    // Touch events with plausible timings
    let touch_count: u32 = rng.gen_range(3..8);
    let mut touch_events = String::new();
    let mut t = timestamp - rng.gen_range(2000..5000) as u128;
    for _ in 0..touch_count {
        let x: u32 = rng.gen_range(50..screen_w - 50);
        let y: u32 = rng.gen_range(100..screen_h - 100);
        let pressure: f32 = rng.gen_range(0.1..0.9);
        touch_events.push_str(&format!("{},{},{},{:.2};", t, x, y, pressure));
        t += rng.gen_range(100..800) as u128;
    }

    // Accelerometer data
    let accel_x: f64 = rng.gen_range(-0.5..0.5);
    let accel_y: f64 = rng.gen_range(-9.9..-9.6);
    let accel_z: f64 = rng.gen_range(-0.3..0.3);

    format!(
        "{}|{}|{}|{}|{}|{}x{}|{}|{}|{:.4},{:.4},{:.4}|{}",
        version,
        bundle_id,
        device_model,
        os_version,
        timestamp,
        screen_w,
        screen_h,
        touch_events,
        rng.gen_range(100..999),
        accel_x,
        accel_y,
        accel_z,
        rng.gen_range(10000..99999),
    )
}

fn urldecoding(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""),
                16,
            ) {
                result.push(byte);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&result).to_string()
}
