use super::*;

impl ExecutionContext {
    pub(super) fn execute_parse_lr(&mut self, settings: &ParseLRSettings) -> crate::error::Result<()> {
        let source = self.variables.resolve_input(&settings.input_var);
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

    pub(super) fn execute_parse_regex(&mut self, settings: &ParseRegexSettings) -> crate::error::Result<()> {
        let source = self.variables.resolve_input(&settings.input_var);
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

    pub(super) fn execute_parse_json(&mut self, settings: &ParseJSONSettings) -> crate::error::Result<()> {
        let source = self.variables.resolve_input(&settings.input_var);
        let path = self.variables.interpolate(&settings.json_path);

        let json: serde_json::Value = serde_json::from_str(&source)
            .map_err(|e| crate::error::AppError::Pipeline(format!("Invalid JSON: {}", e)))?;

        // Convert dot notation to JSON pointer: "user.token" -> "/user/token"
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

    pub(super) fn execute_parse_css(&mut self, settings: &ParseCSSSettings) -> crate::error::Result<()> {
        let source = self.variables.resolve_input(&settings.input_var);
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

    pub(super) fn execute_parse_xpath(&mut self, settings: &ParseXPathSettings) -> crate::error::Result<()> {
        let source = self.variables.resolve_input(&settings.input_var);
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

    pub(super) fn execute_parse_cookie(&mut self, settings: &ParseCookieSettings) -> crate::error::Result<()> {
        let source = self.variables.resolve_input(&settings.input_var);
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

    pub(super) fn execute_lambda_parser(&mut self, settings: &LambdaParserSettings) -> crate::error::Result<()> {
        let input = self.variables.resolve_input(&settings.input_var);
        let lambda_expr = self.variables.interpolate(&settings.lambda_expression);

        // Use simple DSL parser for lambda expressions
        // Supports: x => x.split(',')[0], x => x.trim(), etc.
        let result = self.simple_lambda_parser(&input, &lambda_expr);

        self.variables.set_user(&settings.output_var, result, settings.capture);
        Ok(())
    }

    // Simple fallback parser for basic lambda expressions
    fn simple_lambda_parser(&self, input: &str, lambda_expr: &str) -> String {
        // Support simple patterns like: x => x.split(',')[0]
        if let Some(arrow_pos) = lambda_expr.find("=>") {
            let body = lambda_expr[arrow_pos + 2..].trim();
            let mut result = input.to_string();

            // Parse chained operations
            if body.starts_with("x.") || body.starts_with("input.") {
                let operations = body.split('.').skip(1);
                for op in operations {
                    if let Some(paren_pos) = op.find('(') {
                        let method = &op[..paren_pos];
                        let args = &op[paren_pos + 1..op.len() - 1];

                        result = match method {
                            "split" => {
                                let delimiter = args.trim_matches(|c| c == '\'' || c == '"');
                                let parts: Vec<&str> = result.split(delimiter).collect();
                                serde_json::to_string(&parts).unwrap_or(result)
                            }
                            "trim" => result.trim().to_string(),
                            "toUpperCase" => result.to_uppercase(),
                            "toLowerCase" => result.to_lowercase(),
                            "replace" => {
                                let parts: Vec<&str> = args.split(',').collect();
                                if parts.len() == 2 {
                                    let search = parts[0].trim().trim_matches(|c| c == '\'' || c == '"');
                                    let replace = parts[1].trim().trim_matches(|c| c == '\'' || c == '"');
                                    result.replace(search, replace)
                                } else {
                                    result
                                }
                            }
                            _ => result
                        };
                    } else if let Some(bracket_pos) = op.find('[') {
                        // Array indexing: [0], [1], etc.
                        let method = &op[..bracket_pos];
                        if method.is_empty() {
                            let index_str = &op[bracket_pos + 1..op.len() - 1];
                            if let Ok(index) = index_str.parse::<usize>() {
                                // Try to parse as JSON array
                                if let Ok(arr) = serde_json::from_str::<Vec<String>>(&result) {
                                    result = arr.get(index).cloned().unwrap_or_default();
                                } else {
                                    result = String::new();
                                }
                            }
                        }
                    }
                }
            }

            result
        } else {
            input.to_string()
        }
    }

    // ── Unified Parse dispatch ────────────────────────────────────────
    pub(super) fn execute_parse(&mut self, s: &crate::pipeline::block::ParseSettings) -> crate::error::Result<()> {
        use crate::pipeline::block::{ParseMode, ParseLRSettings, ParseRegexSettings, ParseJSONSettings, ParseCSSSettings, ParseXPathSettings, ParseCookieSettings, LambdaParserSettings};
        match &s.parse_mode {
            ParseMode::LR => self.execute_parse_lr(&ParseLRSettings {
                input_var: s.input_var.clone(), left: s.left.clone(), right: s.right.clone(),
                output_var: s.output_var.clone(), capture: s.capture,
                recursive: s.recursive, case_insensitive: s.case_insensitive,
            }),
            ParseMode::Regex => self.execute_parse_regex(&ParseRegexSettings {
                input_var: s.input_var.clone(), pattern: s.pattern.clone(),
                output_format: s.output_format.clone(), output_var: s.output_var.clone(),
                capture: s.capture, multi_line: s.multi_line,
            }),
            ParseMode::Json => self.execute_parse_json(&ParseJSONSettings {
                input_var: s.input_var.clone(), json_path: s.json_path.clone(),
                output_var: s.output_var.clone(), capture: s.capture,
            }),
            ParseMode::Css => self.execute_parse_css(&ParseCSSSettings {
                input_var: s.input_var.clone(), selector: s.selector.clone(),
                attribute: s.attribute.clone(), output_var: s.output_var.clone(),
                capture: s.capture, index: s.index,
            }),
            ParseMode::XPath => self.execute_parse_xpath(&ParseXPathSettings {
                input_var: s.input_var.clone(), xpath: s.xpath.clone(),
                output_var: s.output_var.clone(), capture: s.capture,
            }),
            ParseMode::Cookie => self.execute_parse_cookie(&ParseCookieSettings {
                input_var: s.input_var.clone(), cookie_name: s.cookie_name.clone(),
                output_var: s.output_var.clone(), capture: s.capture,
            }),
            ParseMode::Lambda => self.execute_lambda_parser(&LambdaParserSettings {
                input_var: s.input_var.clone(), lambda_expression: s.lambda_expression.clone(),
                output_var: s.output_var.clone(), capture: s.capture,
            }),
        }
    }
}


