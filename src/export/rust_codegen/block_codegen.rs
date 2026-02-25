use crate::pipeline::block::*;
use super::helpers::*;

pub(super) fn generate_block_code(block: &Block, indent: usize, vars: &mut VarTracker) -> String {
    let pad = "    ".repeat(indent);
    let mut code = String::new();

    match &block.settings {
        BlockSettings::HttpRequest(s) => {
            let method = s.method.to_lowercase();
            code.push_str(&format!("{}let resp = client.{}(\"{}\")\n", pad, method, s.url));
            for (k, v) in &s.headers {
                code.push_str(&format!("{}    .header(\"{}\", \"{}\")\n", pad, escape_str(k), escape_str(v)));
            }
            if !s.custom_cookies.is_empty() {
                code.push_str(&format!("{}    .header(\"Cookie\", \"{}\")\n", pad, escape_str(&s.custom_cookies.replace('\n', "; "))));
            }
            if !s.body.is_empty() && method != "get" {
                code.push_str(&format!("{}    .body(r#\"{}\"#)\n", pad, s.body));
            }
            code.push_str(&format!("{}    .send()\n", pad));
            code.push_str(&format!("{}    .await?;\n\n", pad));
            let var_prefix = if s.response_var.is_empty() { "SOURCE" } else { &s.response_var };
            code.push_str(&format!("{}let status_code = resp.status().as_u16();\n", pad));
            code.push_str(&format!("{}let {} = resp.text().await?;\n", pad, var_name(var_prefix)));
            vars.define(var_prefix);
            vars.define("status_code");
        }
        BlockSettings::ParseLR(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, var_name(&s.output_var)));
            code.push_str(&format!("{}    let left = \"{}\";\n", pad, escape_str(&s.left)));
            code.push_str(&format!("{}    let right = \"{}\";\n", pad, escape_str(&s.right)));
            code.push_str(&format!("{}    if let Some(start) = {}.find(left) {{\n", pad, input));
            code.push_str(&format!("{}        let after = start + left.len();\n", pad));
            code.push_str(&format!("{}        if let Some(end) = {}[after..].find(right) {{\n", pad, input));
            code.push_str(&format!("{}            {}[after..after + end].to_string()\n", pad, input));
            code.push_str(&format!("{}        }} else {{ String::new() }}\n", pad));
            code.push_str(&format!("{}    }} else {{ String::new() }}\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::ParseRegex(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            code.push_str(&format!("{}let re = Regex::new(r\"{}\")?;\n", pad, s.pattern));
            let letkw = vars.let_or_assign(&s.output_var);
            code.push_str(&format!("{}{}{}= re.captures(&{})\n", pad, letkw, var_name(&s.output_var), input));
            code.push_str(&format!("{}    .and_then(|c| c.get(1))\n", pad));
            code.push_str(&format!("{}    .map(|m| m.as_str().to_string())\n", pad));
            code.push_str(&format!("{}    .unwrap_or_default();\n", pad));
        }
        BlockSettings::ParseJSON(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let pointer = if s.json_path.starts_with('/') {
                s.json_path.clone()
            } else {
                format!("/{}", s.json_path.replace('.', "/"))
            };
            code.push_str(&format!("{}let json: Value = serde_json::from_str(&{})?;\n", pad, input));
            let letkw = vars.let_or_assign(&s.output_var);
            code.push_str(&format!("{}{}{}= json.pointer(\"{}\")\n", pad, letkw, var_name(&s.output_var), pointer));
            code.push_str(&format!("{}    .map(|v| match v {{ Value::String(s) => s.clone(), other => other.to_string() }})\n", pad));
            code.push_str(&format!("{}    .unwrap_or_default();\n", pad));
        }
        BlockSettings::ParseCSS(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            code.push_str(&format!("{}let document = Html::parse_document(&{});\n", pad, input));
            code.push_str(&format!("{}let css_sel = Selector::parse(\"{}\").unwrap();\n", pad, escape_str(&s.selector)));
            let letkw = vars.let_or_assign(&s.output_var);
            if s.attribute == "innerText" || s.attribute.is_empty() {
                code.push_str(&format!("{}{}{}= document.select(&css_sel)\n", pad, letkw, var_name(&s.output_var)));
                code.push_str(&format!("{}    .nth({} as usize)\n", pad, s.index));
                code.push_str(&format!("{}    .map(|el| el.text().collect::<String>())\n", pad));
                code.push_str(&format!("{}    .unwrap_or_default();\n", pad));
            } else {
                code.push_str(&format!("{}{}{}= document.select(&css_sel)\n", pad, letkw, var_name(&s.output_var)));
                code.push_str(&format!("{}    .nth({} as usize)\n", pad, s.index));
                code.push_str(&format!("{}    .and_then(|el| el.value().attr(\"{}\"))\n", pad, escape_str(&s.attribute)));
                code.push_str(&format!("{}    .unwrap_or_default()\n", pad));
                code.push_str(&format!("{}    .to_string();\n", pad));
            }
        }
        BlockSettings::ParseXPath(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            code.push_str(&format!("{}let xpath_pkg = sxd_document::parser::parse(&{}).unwrap();\n", pad, input));
            code.push_str(&format!("{}let xpath_doc = xpath_pkg.as_document();\n", pad));
            code.push_str(&format!("{}let xpath_factory = sxd_xpath::Factory::new();\n", pad));
            code.push_str(&format!("{}let xpath_expr = xpath_factory.build(\"{}\").unwrap().unwrap();\n", pad, escape_str(&s.xpath)));
            code.push_str(&format!("{}let xpath_ctx = sxd_xpath::Context::new();\n", pad));
            code.push_str(&format!("{}let xpath_val = xpath_expr.evaluate(&xpath_ctx, xpath_doc.root()).unwrap();\n", pad));
            code.push_str(&format!("{}{}{}= match xpath_val {{\n", pad, letkw, var_name(&s.output_var)));
            code.push_str(&format!("{}    sxd_xpath::Value::String(s) => s,\n", pad));
            code.push_str(&format!("{}    sxd_xpath::Value::Nodeset(nodes) => {{\n", pad));
            code.push_str(&format!("{}        nodes.document_order().first().map(|n| n.string_value()).unwrap_or_default()\n", pad));
            code.push_str(&format!("{}    }}\n", pad));
            code.push_str(&format!("{}    sxd_xpath::Value::Number(n) => n.to_string(),\n", pad));
            code.push_str(&format!("{}    sxd_xpath::Value::Boolean(b) => b.to_string(),\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::ParseCookie(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            code.push_str(&format!("{}{}{}= {}.split(';')\n", pad, letkw, var_name(&s.output_var), input));
            code.push_str(&format!("{}    .filter_map(|pair| {{\n", pad));
            code.push_str(&format!("{}        let pair = pair.trim();\n", pad));
            code.push_str(&format!("{}        let eq = pair.find('=')?;\n", pad));
            code.push_str(&format!("{}        Some((&pair[..eq], &pair[eq + 1..]))\n", pad));
            code.push_str(&format!("{}    }})\n", pad));
            code.push_str(&format!("{}    .find(|(name, _)| *name == \"{}\")\n", pad, escape_str(&s.cookie_name)));
            code.push_str(&format!("{}    .map(|(_, v)| v.to_string())\n", pad));
            code.push_str(&format!("{}    .unwrap_or_default();\n", pad));
        }
        BlockSettings::KeyCheck(s) => {
            for (i, keychain) in s.keychains.iter().enumerate() {
                let prefix = if i == 0 { "if" } else { "} else if" };
                let conditions: Vec<String> = keychain.conditions.iter()
                    .map(|c| generate_condition_code(c))
                    .collect();
                let cond_str = conditions.join(" && ");
                let status = format!("{:?}", keychain.result).to_uppercase();
                code.push_str(&format!("{}{} {} {{\n", pad, prefix, cond_str));
                code.push_str(&format!("{}    println!(\"{}\");\n", pad, status));
            }
            if !s.keychains.is_empty() {
                code.push_str(&format!("{}}}\n", pad));
            }
        }
        BlockSettings::StringFunction(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            match s.function_type {
                StringFnType::Replace => {
                    code.push_str(&format!("{}{}{}= {}.replace(\"{}\", \"{}\");\n",
                        pad, letkw, vn, input, escape_str(&s.param1), escape_str(&s.param2)));
                }
                StringFnType::ToLower => {
                    code.push_str(&format!("{}{}{}= {}.to_lowercase();\n", pad, letkw, vn, input));
                }
                StringFnType::ToUpper => {
                    code.push_str(&format!("{}{}{}= {}.to_uppercase();\n", pad, letkw, vn, input));
                }
                StringFnType::Trim => {
                    code.push_str(&format!("{}{}{}= {}.trim().to_string();\n", pad, letkw, vn, input));
                }
                StringFnType::Substring => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let start: usize = \"{}\".parse().unwrap_or(0);\n", pad, escape_str(&s.param1)));
                    code.push_str(&format!("{}    let len: usize = \"{}\".parse().unwrap_or({}.len());\n", pad, escape_str(&s.param2), input));
                    code.push_str(&format!("{}    {}.chars().skip(start).take(len).collect::<String>()\n", pad, input));
                    code.push_str(&format!("{}}};\n", pad));
                }
                StringFnType::URLEncode => {
                    code.push_str(&format!("{}{}{}= urlencoding::encode(&{}).to_string();\n", pad, letkw, vn, input));
                }
                StringFnType::URLDecode => {
                    code.push_str(&format!("{}{}{}= urlencoding::decode(&{}).unwrap_or_default().to_string();\n", pad, letkw, vn, input));
                }
                StringFnType::Base64Encode => {
                    code.push_str(&format!("{}{}{}= base64::engine::general_purpose::STANDARD.encode({}.as_bytes());\n", pad, letkw, vn, input));
                }
                StringFnType::Base64Decode => {
                    code.push_str(&format!("{}{}{}= String::from_utf8(base64::engine::general_purpose::STANDARD.decode(&{}).unwrap_or_default()).unwrap_or_default();\n", pad, letkw, vn, input));
                }
                StringFnType::HTMLEntityEncode => {
                    code.push_str(&format!("{}{}{}= {}.replace('&', \"&amp;\").replace('<', \"&lt;\").replace('>', \"&gt;\").replace('\"', \"&quot;\");\n", pad, letkw, vn, input));
                }
                StringFnType::HTMLEntityDecode => {
                    code.push_str(&format!("{}{}{}= {}.replace(\"&amp;\", \"&\").replace(\"&lt;\", \"<\").replace(\"&gt;\", \">\").replace(\"&quot;\", \"\\\"\");\n", pad, letkw, vn, input));
                }
                StringFnType::Split => {
                    code.push_str(&format!("{}{}{}= {}.split(\"{}\").collect::<Vec<_>>().join(\"\\n\");\n", pad, letkw, vn, input, escape_str(&s.param1)));
                }
                StringFnType::Reverse => {
                    code.push_str(&format!("{}{}{}= {}.chars().rev().collect::<String>();\n", pad, letkw, vn, input));
                }
                StringFnType::Length => {
                    code.push_str(&format!("{}{}{}= {}.len().to_string();\n", pad, letkw, vn, input));
                }
                StringFnType::RandomString => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let len: usize = \"{}\".parse().unwrap_or(16);\n", pad, escape_str(&s.param1)));
                    code.push_str(&format!("{}    use rand::Rng;\n", pad));
                    code.push_str(&format!("{}    let mut rng = rand::thread_rng();\n", pad));
                    code.push_str(&format!("{}    (0..len).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect::<String>()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
            }
        }
        BlockSettings::ListFunction(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            match s.function_type {
                ListFnType::Join => {
                    let sep = if s.param1.is_empty() { ", " } else { &s.param1 };
                    code.push_str(&format!("{}{}{}= {}.lines().collect::<Vec<_>>().join(\"{}\");\n", pad, letkw, vn, input, escape_str(sep)));
                }
                ListFnType::Sort => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let mut items: Vec<&str> = {}.lines().collect();\n", pad, input));
                    code.push_str(&format!("{}    items.sort();\n", pad));
                    code.push_str(&format!("{}    items.join(\"\\n\")\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                ListFnType::Shuffle => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    use rand::seq::SliceRandom;\n", pad));
                    code.push_str(&format!("{}    let mut items: Vec<&str> = {}.lines().collect();\n", pad, input));
                    code.push_str(&format!("{}    items.shuffle(&mut rand::thread_rng());\n", pad));
                    code.push_str(&format!("{}    items.join(\"\\n\")\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                ListFnType::Add => {
                    code.push_str(&format!("{}{}{}= format!(\"{{}}\\n{}\", {});\n", pad, letkw, vn, escape_str(&s.param1), input));
                }
                ListFnType::Remove => {
                    code.push_str(&format!("{}{}{}= {}.lines().filter(|l| *l != \"{}\").collect::<Vec<_>>().join(\"\\n\");\n", pad, letkw, vn, input, escape_str(&s.param1)));
                }
                ListFnType::Deduplicate => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let mut seen = std::collections::HashSet::new();\n", pad));
                    code.push_str(&format!("{}    {}.lines().filter(|l| seen.insert(l.to_string())).collect::<Vec<_>>().join(\"\\n\")\n", pad, input));
                    code.push_str(&format!("{}}};\n", pad));
                }
                ListFnType::RandomItem => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    use rand::seq::SliceRandom;\n", pad));
                    code.push_str(&format!("{}    let items: Vec<&str> = {}.lines().collect();\n", pad, input));
                    code.push_str(&format!("{}    items.choose(&mut rand::thread_rng()).unwrap_or(&\"\").to_string()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                ListFnType::Length => {
                    code.push_str(&format!("{}{}{}= {}.lines().count().to_string();\n", pad, letkw, vn, input));
                }
            }
        }
        BlockSettings::CryptoFunction(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            match s.function_type {
                CryptoFnType::MD5 => {
                    code.push_str(&format!("{}{}{}= format!(\"{{:x}}\", md5::compute({}.as_bytes()));\n", pad, letkw, vn, input));
                }
                CryptoFnType::SHA1 => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let mut hasher = sha1::Sha1::new();\n", pad));
                    code.push_str(&format!("{}    hasher.update({}.as_bytes());\n", pad, input));
                    code.push_str(&format!("{}    format!(\"{{:x}}\", hasher.finalize())\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                CryptoFnType::SHA256 => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let mut hasher = sha2::Sha256::new();\n", pad));
                    code.push_str(&format!("{}    hasher.update({}.as_bytes());\n", pad, input));
                    code.push_str(&format!("{}    format!(\"{{:x}}\", hasher.finalize())\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                CryptoFnType::SHA384 => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let mut hasher = sha2::Sha384::new();\n", pad));
                    code.push_str(&format!("{}    hasher.update({}.as_bytes());\n", pad, input));
                    code.push_str(&format!("{}    format!(\"{{:x}}\", hasher.finalize())\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                CryptoFnType::SHA512 => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let mut hasher = sha2::Sha512::new();\n", pad));
                    code.push_str(&format!("{}    hasher.update({}.as_bytes());\n", pad, input));
                    code.push_str(&format!("{}    format!(\"{{:x}}\", hasher.finalize())\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                CryptoFnType::CRC32 => {
                    code.push_str(&format!("{}{}{}= format!(\"{{}}\", crc32fast::hash({}.as_bytes()));\n", pad, letkw, vn, input));
                }
                CryptoFnType::HMACSHA256 => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    use hmac::{{Hmac, Mac}};\n", pad));
                    code.push_str(&format!("{}    type HmacSha256 = Hmac<sha2::Sha256>;\n", pad));
                    code.push_str(&format!("{}    let mut mac = HmacSha256::new_from_slice(\"{}\".as_bytes()).unwrap();\n", pad, escape_str(&s.key)));
                    code.push_str(&format!("{}    mac.update({}.as_bytes());\n", pad, input));
                    code.push_str(&format!("{}    format!(\"{{:x}}\", mac.finalize().into_bytes())\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                CryptoFnType::HMACSHA512 => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    use hmac::{{Hmac, Mac}};\n", pad));
                    code.push_str(&format!("{}    type HmacSha512 = Hmac<sha2::Sha512>;\n", pad));
                    code.push_str(&format!("{}    let mut mac = HmacSha512::new_from_slice(\"{}\".as_bytes()).unwrap();\n", pad, escape_str(&s.key)));
                    code.push_str(&format!("{}    mac.update({}.as_bytes());\n", pad, input));
                    code.push_str(&format!("{}    format!(\"{{:x}}\", mac.finalize().into_bytes())\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                CryptoFnType::HMACMD5 => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    use hmac::{{Hmac, Mac}};\n", pad));
                    code.push_str(&format!("{}    type HmacMd5 = Hmac<md5::Md5>;\n", pad));
                    code.push_str(&format!("{}    let mut mac = HmacMd5::new_from_slice(\"{}\".as_bytes()).unwrap();\n", pad, escape_str(&s.key)));
                    code.push_str(&format!("{}    mac.update({}.as_bytes());\n", pad, input));
                    code.push_str(&format!("{}    format!(\"{{:x}}\", mac.finalize().into_bytes())\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                CryptoFnType::Base64Encode => {
                    code.push_str(&format!("{}{}{}= base64::engine::general_purpose::STANDARD.encode({}.as_bytes());\n", pad, letkw, vn, input));
                }
                CryptoFnType::Base64Decode => {
                    code.push_str(&format!("{}{}{}= String::from_utf8(base64::engine::general_purpose::STANDARD.decode(&{}).unwrap_or_default()).unwrap_or_default();\n", pad, letkw, vn, input));
                }
                CryptoFnType::BCryptHash => {
                    code.push_str(&format!("{}{}{}= bcrypt::hash(&{}, bcrypt::DEFAULT_COST).unwrap_or_default();\n", pad, letkw, vn, input));
                }
                CryptoFnType::BCryptVerify => {
                    code.push_str(&format!("{}{}{}= bcrypt::verify(&{}, \"{}\").unwrap_or(false).to_string();\n", pad, letkw, vn, input, escape_str(&s.key)));
                }
                CryptoFnType::AESEncrypt | CryptoFnType::AESDecrypt => {
                    code.push_str(&format!("{}// TODO: AES encrypt/decrypt requires additional setup (IV, mode)\n", pad));
                    code.push_str(&format!("{}{}{}= String::new();\n", pad, letkw, vn));
                }
            }
        }
        BlockSettings::ConversionFunction(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            let from = s.from_type.as_str();
            let to = s.to_type.as_str();
            match (from, to) {
                ("string", "int") => {
                    code.push_str(&format!("{}{}{}= {}.parse::<i64>().unwrap_or(0).to_string();\n", pad, letkw, vn, input));
                }
                ("int", "string") | ("float", "string") => {
                    code.push_str(&format!("{}{}{}= {}.to_string();\n", pad, letkw, vn, input));
                }
                ("string", "float") => {
                    code.push_str(&format!("{}{}{}= {}.parse::<f64>().unwrap_or(0.0).to_string();\n", pad, letkw, vn, input));
                }
                ("hex", "string") => {
                    code.push_str(&format!("{}{}{}= hex::decode(&{}).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();\n", pad, letkw, vn, input));
                }
                ("string", "hex") => {
                    code.push_str(&format!("{}{}{}= hex::encode({}.as_bytes());\n", pad, letkw, vn, input));
                }
                ("base64", "string") => {
                    code.push_str(&format!("{}{}{}= String::from_utf8(base64::engine::general_purpose::STANDARD.decode(&{}).unwrap_or_default()).unwrap_or_default();\n", pad, letkw, vn, input));
                }
                ("string", "base64") => {
                    code.push_str(&format!("{}{}{}= base64::engine::general_purpose::STANDARD.encode({}.as_bytes());\n", pad, letkw, vn, input));
                }
                _ => {
                    code.push_str(&format!("{}{}{}= {}.to_string(); // {}→{}\n", pad, letkw, vn, input, from, to));
                }
            }
        }
        BlockSettings::DateFunction(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            match s.function_type {
                DateFnType::Now => {
                    code.push_str(&format!("{}{}{}= Local::now().format(\"{}\").to_string();\n", pad, letkw, vn, escape_str(&s.format)));
                }
                DateFnType::UnixTimestamp => {
                    code.push_str(&format!("{}{}{}= Utc::now().timestamp().to_string();\n", pad, letkw, vn));
                }
                DateFnType::UnixToDate => {
                    let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let ts: i64 = {}.parse().unwrap_or(0);\n", pad, input));
                    code.push_str(&format!("{}    DateTime::from_timestamp(ts, 0).map(|dt| dt.format(\"{}\").to_string()).unwrap_or_default()\n", pad, escape_str(&s.format)));
                    code.push_str(&format!("{}}};\n", pad));
                }
                DateFnType::FormatDate => {
                    let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    // Parse input date and reformat\n", pad));
                    code.push_str(&format!("{}    NaiveDateTime::parse_from_str(&{}, \"%Y-%m-%d %H:%M:%S\")\n", pad, input));
                    code.push_str(&format!("{}        .map(|dt| dt.format(\"{}\").to_string())\n", pad, escape_str(&s.format)));
                    code.push_str(&format!("{}        .unwrap_or_default()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                DateFnType::ParseDate => {
                    let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
                    code.push_str(&format!("{}{}{}= NaiveDateTime::parse_from_str(&{}, \"{}\").map(|d| d.to_string()).unwrap_or_default();\n",
                        pad, letkw, vn, input, escape_str(&s.format)));
                }
                DateFnType::AddTime | DateFnType::SubtractTime => {
                    let op = if matches!(s.function_type, DateFnType::AddTime) { "+" } else { "-" };
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let now = Local::now();\n", pad));
                    let dur = match s.unit.as_str() {
                        "seconds" => format!("chrono::Duration::seconds({})", s.amount),
                        "minutes" => format!("chrono::Duration::minutes({})", s.amount),
                        "hours" => format!("chrono::Duration::hours({})", s.amount),
                        "days" => format!("chrono::Duration::days({})", s.amount),
                        _ => format!("chrono::Duration::seconds({})", s.amount),
                    };
                    code.push_str(&format!("{}    (now {} {}).format(\"{}\").to_string()\n", pad, op, dur, escape_str(&s.format)));
                    code.push_str(&format!("{}}};\n", pad));
                }
            }
        }
        BlockSettings::CaseSwitch(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}{}{}= match {}.as_str() {{\n", pad, letkw, vn, input));
            for case in &s.cases {
                code.push_str(&format!("{}    \"{}\" => \"{}\".to_string(),\n", pad, escape_str(&case.match_value), escape_str(&case.result_value)));
            }
            code.push_str(&format!("{}    _ => \"{}\".to_string(),\n", pad, escape_str(&s.default_value)));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::IfElse(s) => {
            let cond = generate_condition_code(&s.condition);
            code.push_str(&format!("{}if {} {{\n", pad, cond));
            for child in &s.true_blocks {
                if !child.disabled {
                    code.push_str(&generate_block_code(child, indent + 1, vars));
                }
            }
            if !s.false_blocks.is_empty() {
                code.push_str(&format!("{}}} else {{\n", pad));
                for child in &s.false_blocks {
                    if !child.disabled {
                        code.push_str(&generate_block_code(child, indent + 1, vars));
                    }
                }
            }
            code.push_str(&format!("{}}}\n", pad));
        }
        BlockSettings::Loop(s) => {
            match s.loop_type {
                LoopType::ForEach => {
                    let input = if vars.is_defined(&s.list_var) { var_name(&s.list_var) } else { "source".into() };
                    let item = var_name(&s.item_var);
                    code.push_str(&format!("{}for {} in {}.lines() {{\n", pad, item, input));
                    vars.define(&s.item_var);
                    for child in &s.blocks {
                        if !child.disabled {
                            code.push_str(&generate_block_code(child, indent + 1, vars));
                        }
                    }
                    code.push_str(&format!("{}}}\n", pad));
                }
                LoopType::Repeat => {
                    code.push_str(&format!("{}for _i in 0..{} {{\n", pad, s.count));
                    for child in &s.blocks {
                        if !child.disabled {
                            code.push_str(&generate_block_code(child, indent + 1, vars));
                        }
                    }
                    code.push_str(&format!("{}}}\n", pad));
                }
            }
        }
        BlockSettings::Script(s) => {
            generate_script_block(&s.code, &s.output_var, &pad, vars, &mut code);
        }
        BlockSettings::CookieContainer(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            if s.source_type == "file" {
                code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                code.push_str(&format!("{}    let content = std::fs::read_to_string(\"{}\")?;\n", pad, escape_str(&s.source)));
                code.push_str(&format!("{}    // Parse Netscape cookie format\n", pad));
                code.push_str(&format!("{}    content.lines()\n", pad));
                code.push_str(&format!("{}        .filter(|l| !l.starts_with('#') && !l.is_empty())\n", pad));
                code.push_str(&format!("{}        .filter_map(|l| {{\n", pad));
                code.push_str(&format!("{}            let parts: Vec<&str> = l.split('\\t').collect();\n", pad));
                code.push_str(&format!("{}            if parts.len() >= 7 {{ Some(format!(\"{{}}={{}}\", parts[5], parts[6])) }} else {{ None }}\n", pad));
                code.push_str(&format!("{}        }})\n", pad));
                code.push_str(&format!("{}        .collect::<Vec<_>>().join(\"; \")\n", pad));
                code.push_str(&format!("{}}};\n", pad));
            } else {
                code.push_str(&format!("{}{}{}= \"{}\".to_string();\n", pad, letkw, vn, escape_str(&s.source)));
            }
        }
        BlockSettings::Webhook(s) => {
            let method = s.method.to_lowercase();
            code.push_str(&format!("{}let webhook_resp = client.{}(\"{}\")\n", pad, method, s.url));
            for (k, v) in &s.headers {
                code.push_str(&format!("{}    .header(\"{}\", \"{}\")\n", pad, escape_str(k), escape_str(v)));
            }
            if !s.body_template.is_empty() {
                code.push_str(&format!("{}    .body(r#\"{}\"#)\n", pad, s.body_template));
            }
            code.push_str(&format!("{}    .send()\n", pad));
            code.push_str(&format!("{}    .await?;\n", pad));
        }
        BlockSettings::SetVariable(s) => {
            let letkw = vars.let_or_assign(&s.name);
            code.push_str(&format!("{}{}{}= \"{}\".to_string();\n", pad, letkw, var_name(&s.name), escape_str(&s.value)));
        }
        BlockSettings::Delay(s) => {
            if s.min_ms == s.max_ms {
                code.push_str(&format!("{}tokio::time::sleep(std::time::Duration::from_millis({})).await;\n", pad, s.min_ms));
            } else {
                code.push_str(&format!("{}tokio::time::sleep(std::time::Duration::from_millis(rand::Rng::gen_range(&mut rand::thread_rng(), {}..={}))).await;\n", pad, s.min_ms, s.max_ms));
            }
        }
        BlockSettings::Log(s) => {
            code.push_str(&format!("{}println!(\"{}\");\n", pad, escape_str(&s.message)));
        }
        BlockSettings::ClearCookies => {
            code.push_str(&format!("{}// Clear session cookies — rebuild client with fresh cookie store\n", pad));
            code.push_str(&format!("{}client = Client::builder()\n", pad));
            code.push_str(&format!("{}    .emulation(emulation)\n", pad));
            code.push_str(&format!("{}    .cookie_store(true)\n", pad));
            code.push_str(&format!("{}    .build()?;\n", pad));
        }
        // Browser automation
        BlockSettings::BrowserOpen(s) => {
            code.push_str(&format!("{}let (browser, mut handler) = Browser::launch(\n", pad));
            code.push_str(&format!("{}    BrowserConfig::builder()\n", pad));
            if s.headless {
                code.push_str(&format!("{}        .with_head()\n", pad));
            }
            code.push_str(&format!("{}        .build()\n", pad));
            code.push_str(&format!("{}        .map_err(|e| format!(\"Browser config error: {{}}\", e))?\n", pad));
            code.push_str(&format!("{}).await?;\n", pad));
            code.push_str(&format!("{}tokio::spawn(async move {{ while handler.next().await.is_some() {{}} }});\n", pad));
        }
        BlockSettings::NavigateTo(s) => {
            code.push_str(&format!("{}let page = browser.new_page(\"{}\").await?;\n", pad, escape_str(&s.url)));
            code.push_str(&format!("{}page.wait_for_navigation().await?;\n", pad));
            code.push_str(&format!("{}let source = page.content().await?;\n", pad));
            vars.define("source");
        }
        BlockSettings::ClickElement(s) => {
            code.push_str(&format!("{}page.find_element(\"{}\").await?.click().await?;\n", pad, escape_str(&s.selector)));
            if s.wait_for_navigation {
                code.push_str(&format!("{}page.wait_for_navigation().await?;\n", pad));
            }
        }
        BlockSettings::TypeText(s) => {
            if s.clear_first {
                code.push_str(&format!("{}// Clear field first\n", pad));
                code.push_str(&format!("{}page.find_element(\"{}\").await?.click().await?;\n", pad, escape_str(&s.selector)));
                code.push_str(&format!("{}page.execute(chromiumoxide::cdp::browser_protocol::input::DispatchKeyEventParams::builder().r#type(chromiumoxide::cdp::browser_protocol::input::DispatchKeyEventType::KeyDown).key(\"a\").code(\"KeyA\").modifiers(2).build().unwrap()).await?;\n", pad));
            }
            code.push_str(&format!("{}page.find_element(\"{}\").await?.type_str(\"{}\").await?;\n", pad, escape_str(&s.selector), escape_str(&s.text)));
        }
        BlockSettings::WaitForElement(s) => {
            code.push_str(&format!("{}// Wait for element: {} (state: {})\n", pad, s.selector, s.state));
            code.push_str(&format!("{}let start = std::time::Instant::now();\n", pad));
            code.push_str(&format!("{}while start.elapsed().as_millis() < {} {{\n", pad, s.timeout_ms));
            code.push_str(&format!("{}    if page.find_element(\"{}\").await.is_ok() {{ break; }}\n", pad, escape_str(&s.selector)));
            code.push_str(&format!("{}    tokio::time::sleep(std::time::Duration::from_millis(100)).await;\n", pad));
            code.push_str(&format!("{}}}\n", pad));
        }
        BlockSettings::GetElementText(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            if s.attribute.is_empty() {
                code.push_str(&format!("{}{}{}= page.find_element(\"{}\").await?.inner_text().await?.unwrap_or_default();\n",
                    pad, letkw, vn, escape_str(&s.selector)));
            } else {
                code.push_str(&format!("{}{}{}= page.find_element(\"{}\").await?.attribute(\"{}\").await?.unwrap_or_default();\n",
                    pad, letkw, vn, escape_str(&s.selector), escape_str(&s.attribute)));
            }
        }
        BlockSettings::Screenshot(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}let screenshot_bytes = page.screenshot(\n", pad));
            code.push_str(&format!("{}    chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams::builder()\n", pad));
            code.push_str(&format!("{}        .build()\n", pad));
            code.push_str(&format!("{}).await?;\n", pad));
            code.push_str(&format!("{}{}{}= base64::engine::general_purpose::STANDARD.encode(&screenshot_bytes);\n", pad, letkw, vn));
        }
        BlockSettings::ExecuteJs(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}{}{}= page.evaluate_expression(r#\"{}\"#).await?.into_value::<String>().unwrap_or_default();\n",
                pad, letkw, vn, s.code));
        }
        BlockSettings::RandomUserAgent(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    use rand::seq::SliceRandom;\n", pad));
            code.push_str(&format!("{}    let agents = vec![\n", pad));
            code.push_str(&format!("{}        \"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36\",\n", pad));
            code.push_str(&format!("{}        \"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36\",\n", pad));
            code.push_str(&format!("{}        \"Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0\",\n", pad));
            code.push_str(&format!("{}    ];\n", pad));
            code.push_str(&format!("{}    agents.choose(&mut rand::thread_rng()).unwrap().to_string()\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::OcrCaptcha(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}// OCR Captcha: requires rusty-tesseract and Tesseract 4+ installed\n", pad));
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    let img = rusty_tesseract::Image::from_dynamic_image(&image::load_from_memory(&base64::engine::general_purpose::STANDARD.decode(&{}).unwrap()).unwrap()).unwrap();\n", pad, var_name(&s.input_var)));
            code.push_str(&format!("{}    let args = rusty_tesseract::Args {{ lang: \"{}\".to_string(), psm: Some({}), ..Default::default() }};\n", pad, escape_str(&s.language), s.psm));
            code.push_str(&format!("{}    rusty_tesseract::image_to_string(&img, &args).unwrap_or_default().trim().to_string()\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::RecaptchaInvisible(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}// reCAPTCHA Invisible solver\n", pad));
            code.push_str(&format!("{}let anchor_resp = client.get(\"{}\")\n", pad, escape_str(&s.anchor_url)));
            code.push_str(&format!("{}    .header(\"User-Agent\", \"{}\")\n", pad, escape_str(&s.user_agent)));
            code.push_str(&format!("{}    .send().await?.text().await?;\n", pad));
            code.push_str(&format!("{}let token = anchor_resp.split(\"recaptcha-token\\\" value=\\\"\").nth(1)\n", pad));
            code.push_str(&format!("{}    .and_then(|s| s.split('\"').next()).unwrap_or_default().to_string();\n", pad));
            code.push_str(&format!("{}let reload_body = format!(\"v={}&reason=q&c={{}}&k={}&co={}&size={}\", token);\n",
                pad, escape_str(&s.v), escape_str(&s.sitekey), escape_str(&s.co), escape_str(&s.size)));
            code.push_str(&format!("{}let reload_resp = client.post(\"{}\")\n", pad, escape_str(&s.reload_url)));
            code.push_str(&format!("{}    .header(\"Content-Type\", \"application/x-www-form-urlencoded\")\n", pad));
            code.push_str(&format!("{}    .body(reload_body).send().await?.text().await?;\n", pad));
            code.push_str(&format!("{}{}{}= reload_resp.split(\"\\\"rresp\\\",\\\"\").nth(1)\n", pad, letkw, vn));
            code.push_str(&format!("{}    .and_then(|s| s.split('\"').next()).unwrap_or_default().to_string();\n", pad));
        }
        BlockSettings::XacfSensor(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}// XACF Sensor Data generation\n", pad));
            code.push_str(&format!("{}{}{}= format!(\n", pad, letkw, vn));
            code.push_str(&format!("{}    \"{}|{}|iPhone14,3|18.1|{{}}|1170x2532||{{}}|0.0,-9.8,0.0|{{}}\",\n", pad, escape_str(&s.version), escape_str(&s.bundle_id)));
            code.push_str(&format!("{}    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),\n", pad));
            code.push_str(&format!("{}    rand::Rng::gen_range(&mut rand::thread_rng(), 100..999),\n", pad));
            code.push_str(&format!("{}    rand::Rng::gen_range(&mut rand::thread_rng(), 10000..99999),\n", pad));
            code.push_str(&format!("{});\n", pad));
        }
        BlockSettings::RandomData(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}// Random Data: {:?}\n", pad, s.data_type));
            match s.data_type {
                RandomDataType::String => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let mut rng = rand::thread_rng();\n", pad));
                    code.push_str(&format!("{}    (0..{}).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect::<String>()\n", pad, s.string_length));
                    code.push_str(&format!("{}}};\n", pad));
                }
                RandomDataType::Uuid => {
                    code.push_str(&format!("{}{}{}= uuid::Uuid::new_v4().to_string();\n", pad, letkw, vn));
                }
                RandomDataType::Number => {
                    code.push_str(&format!("{}{}{}= rand::Rng::gen_range(&mut rand::thread_rng(), {}..={}).to_string();\n", pad, letkw, vn, s.number_min, s.number_max));
                }
                RandomDataType::Email => {
                    code.push_str(&format!("{}{}{}= format!(\"{{}}@gmail.com\", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());\n", pad, letkw, vn));
                }
                _ => {
                    code.push_str(&format!("{}{}{}= String::from(\"TODO: random {:?}\");\n", pad, letkw, vn, s.data_type));
                }
            }
        }
        BlockSettings::DataDomeSensor(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}// DataDome sensor data generation\n", pad));
            code.push_str(&format!("{}// Credit: glizzykingdreko/datadome-wasm\n", pad));
            code.push_str(&format!("{}{}{}= String::from(\"TODO: DataDome sensor generation for {}\");\n", pad, letkw, vn, escape_str(&s.site_url)));
        }
        BlockSettings::Plugin(s) => {
            code.push_str(&format!("{}// Plugin block: {} (requires DLL plugin at runtime)\n", pad, s.plugin_block_type));
            if !s.output_var.is_empty() {
                let letkw = vars.let_or_assign(&s.output_var);
                let vn = var_name(&s.output_var);
                code.push_str(&format!("{}{}{}= String::new(); // Plugin output\n", pad, letkw, vn));
            }
        }
        BlockSettings::AkamaiV3Sensor(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}// Akamai V3 Sensor Data (credit: glizzykingdreko)\n", pad));
            code.push_str(&format!("{}// https://github.com/glizzykingdreko/akamai-v3-sensor-data-helper\n", pad));
            code.push_str(&format!("{}{}{}= String::from(\"TODO: Akamai V3 {:?} mode\");\n", pad, letkw, vn, s.mode));
        }
        BlockSettings::Group(s) => {
            code.push_str(&format!("{}// Group: {}\n", pad, block.label));
            for child in &s.blocks {
                if !child.disabled {
                    code.push_str(&generate_block_code(child, indent, vars));
                }
            }
        }
        BlockSettings::TcpRequest(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    let mut stream = tokio::net::TcpStream::connect(\"{}:{}\").await?;\n", pad, escape_str(&s.host), s.port));
            if !s.data.is_empty() {
                code.push_str(&format!("{}    stream.write_all(b\"{}\").await?;\n", pad, escape_str(&s.data)));
                code.push_str(&format!("{}    stream.flush().await?;\n", pad));
            }
            code.push_str(&format!("{}    let mut buf = vec![0u8; 65536];\n", pad));
            code.push_str(&format!("{}    let n = stream.read(&mut buf).await.unwrap_or(0);\n", pad));
            code.push_str(&format!("{}    String::from_utf8_lossy(&buf[..n]).to_string()\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::UdpRequest(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    let socket = tokio::net::UdpSocket::bind(\"0.0.0.0:0\").await?;\n", pad));
            code.push_str(&format!("{}    socket.send_to(b\"{}\", \"{}:{}\").await?;\n", pad, escape_str(&s.data), escape_str(&s.host), s.port));
            code.push_str(&format!("{}    let mut buf = vec![0u8; 65536];\n", pad));
            code.push_str(&format!("{}    let (n, _) = socket.recv_from(&mut buf).await?;\n", pad));
            code.push_str(&format!("{}    String::from_utf8_lossy(&buf[..n]).to_string()\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::FtpRequest(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    let stream = tokio::net::TcpStream::connect(\"{}:{}\").await?;\n", pad, escape_str(&s.host), s.port));
            code.push_str(&format!("{}    let (r, mut w) = tokio::io::split(stream);\n", pad));
            code.push_str(&format!("{}    let mut r = tokio::io::BufReader::new(r);\n", pad));
            code.push_str(&format!("{}    let mut transcript = String::new();\n", pad));
            code.push_str(&format!("{}    let mut line = String::new();\n", pad));
            code.push_str(&format!("{}    r.read_line(&mut line).await?; // banner\n", pad));
            code.push_str(&format!("{}    transcript.push_str(&line);\n", pad));
            code.push_str(&format!("{}    for cmd in [\"USER {}\", \"PASS {}\", \"{}\", \"QUIT\"] {{\n",
                pad, escape_str(&s.username), escape_str(&s.password), escape_str(&s.command)));
            code.push_str(&format!("{}        w.write_all(format!(\"{{}}\\r\\n\", cmd).as_bytes()).await?;\n", pad));
            code.push_str(&format!("{}        w.flush().await?;\n", pad));
            code.push_str(&format!("{}        line.clear();\n", pad));
            code.push_str(&format!("{}        r.read_line(&mut line).await?;\n", pad));
            code.push_str(&format!("{}        transcript.push_str(&line);\n", pad));
            code.push_str(&format!("{}    }}\n", pad));
            code.push_str(&format!("{}    transcript\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::SshRequest(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    let mut stream = tokio::net::TcpStream::connect(\"{}:{}\").await?;\n", pad, escape_str(&s.host), s.port));
            code.push_str(&format!("{}    let mut buf = vec![0u8; 4096];\n", pad));
            code.push_str(&format!("{}    let n = stream.read(&mut buf).await?;\n", pad));
            code.push_str(&format!("{}    let banner = String::from_utf8_lossy(&buf[..n]).to_string();\n", pad));
            code.push_str(&format!("{}    stream.write_all(b\"SSH-2.0-ReqFlow_1.0\\r\\n\").await?;\n", pad));
            code.push_str(&format!("{}    banner // Note: full SSH auth requires ssh2 crate\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::ImapRequest(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}// IMAP connection to {}:{}\n", pad, escape_str(&s.host), s.port));
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    let stream = tokio::net::TcpStream::connect(\"{}:{}\").await?;\n", pad, escape_str(&s.host), s.port));
            if s.use_tls {
                code.push_str(&format!("{}    let connector = native_tls::TlsConnector::new()?;\n", pad));
                code.push_str(&format!("{}    let connector = tokio_native_tls::TlsConnector::from(connector);\n", pad));
                code.push_str(&format!("{}    let stream = connector.connect(\"{}\", stream).await?;\n", pad, escape_str(&s.host)));
            }
            code.push_str(&format!("{}    let (r, mut w) = tokio::io::split(stream);\n", pad));
            code.push_str(&format!("{}    let mut r = tokio::io::BufReader::new(r);\n", pad));
            code.push_str(&format!("{}    let mut line = String::new(); let mut transcript = String::new();\n", pad));
            code.push_str(&format!("{}    r.read_line(&mut line).await?; transcript.push_str(&line); // banner\n", pad));
            code.push_str(&format!("{}    w.write_all(b\"a001 LOGIN {} {}\\r\\n\").await?; w.flush().await?;\n",
                pad, escape_str(&s.username), escape_str(&s.password)));
            code.push_str(&format!("{}    line.clear(); r.read_line(&mut line).await?; transcript.push_str(&line);\n", pad));
            if !s.command.is_empty() && s.command != "LOGIN" {
                code.push_str(&format!("{}    w.write_all(b\"a002 {}\\r\\n\").await?; w.flush().await?;\n", pad, escape_str(&s.command)));
                code.push_str(&format!("{}    line.clear(); r.read_line(&mut line).await?; transcript.push_str(&line);\n", pad));
            }
            code.push_str(&format!("{}    w.write_all(b\"a003 LOGOUT\\r\\n\").await?;\n", pad));
            code.push_str(&format!("{}    transcript\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::SmtpRequest(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}// SMTP connection to {}:{}\n", pad, escape_str(&s.host), s.port));
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    let stream = tokio::net::TcpStream::connect(\"{}:{}\").await?;\n", pad, escape_str(&s.host), s.port));
            if s.use_tls {
                code.push_str(&format!("{}    let connector = native_tls::TlsConnector::new()?;\n", pad));
                code.push_str(&format!("{}    let connector = tokio_native_tls::TlsConnector::from(connector);\n", pad));
                code.push_str(&format!("{}    let stream = connector.connect(\"{}\", stream).await?;\n", pad, escape_str(&s.host)));
            }
            code.push_str(&format!("{}    let (r, mut w) = tokio::io::split(stream);\n", pad));
            code.push_str(&format!("{}    let mut r = tokio::io::BufReader::new(r);\n", pad));
            code.push_str(&format!("{}    let mut line = String::new(); let mut transcript = String::new();\n", pad));
            code.push_str(&format!("{}    r.read_line(&mut line).await?; transcript.push_str(&line);\n", pad));
            code.push_str(&format!("{}    w.write_all(b\"EHLO ironbullet\\r\\n\").await?; w.flush().await?;\n", pad));
            code.push_str(&format!("{}    line.clear(); r.read_line(&mut line).await?; transcript.push_str(&line);\n", pad));
            if !s.username.is_empty() {
                code.push_str(&format!("{}    w.write_all(b\"AUTH LOGIN\\r\\n\").await?; w.flush().await?;\n", pad));
                code.push_str(&format!("{}    line.clear(); r.read_line(&mut line).await?; transcript.push_str(&line);\n", pad));
                code.push_str(&format!("{}    w.write_all(format!(\"{{}}\\r\\n\", base64::engine::general_purpose::STANDARD.encode(\"{}\")).as_bytes()).await?;\n", pad, escape_str(&s.username)));
                code.push_str(&format!("{}    w.flush().await?; line.clear(); r.read_line(&mut line).await?; transcript.push_str(&line);\n", pad));
                code.push_str(&format!("{}    w.write_all(format!(\"{{}}\\r\\n\", base64::engine::general_purpose::STANDARD.encode(\"{}\")).as_bytes()).await?;\n", pad, escape_str(&s.password)));
                code.push_str(&format!("{}    w.flush().await?; line.clear(); r.read_line(&mut line).await?; transcript.push_str(&line);\n", pad));
            }
            code.push_str(&format!("{}    w.write_all(b\"QUIT\\r\\n\").await?;\n", pad));
            code.push_str(&format!("{}    transcript\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::PopRequest(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}// POP3 connection to {}:{}\n", pad, escape_str(&s.host), s.port));
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    let stream = tokio::net::TcpStream::connect(\"{}:{}\").await?;\n", pad, escape_str(&s.host), s.port));
            if s.use_tls {
                code.push_str(&format!("{}    let connector = native_tls::TlsConnector::new()?;\n", pad));
                code.push_str(&format!("{}    let connector = tokio_native_tls::TlsConnector::from(connector);\n", pad));
                code.push_str(&format!("{}    let stream = connector.connect(\"{}\", stream).await?;\n", pad, escape_str(&s.host)));
            }
            code.push_str(&format!("{}    let (r, mut w) = tokio::io::split(stream);\n", pad));
            code.push_str(&format!("{}    let mut r = tokio::io::BufReader::new(r);\n", pad));
            code.push_str(&format!("{}    let mut line = String::new(); let mut transcript = String::new();\n", pad));
            code.push_str(&format!("{}    r.read_line(&mut line).await?; transcript.push_str(&line);\n", pad));
            code.push_str(&format!("{}    w.write_all(b\"USER {}\\r\\n\").await?; w.flush().await?;\n", pad, escape_str(&s.username)));
            code.push_str(&format!("{}    line.clear(); r.read_line(&mut line).await?; transcript.push_str(&line);\n", pad));
            code.push_str(&format!("{}    w.write_all(b\"PASS {}\\r\\n\").await?; w.flush().await?;\n", pad, escape_str(&s.password)));
            code.push_str(&format!("{}    line.clear(); r.read_line(&mut line).await?; transcript.push_str(&line);\n", pad));
            if !s.command.is_empty() {
                code.push_str(&format!("{}    w.write_all(b\"{}\\r\\n\").await?; w.flush().await?;\n", pad, escape_str(&s.command)));
                code.push_str(&format!("{}    line.clear(); r.read_line(&mut line).await?; transcript.push_str(&line);\n", pad));
            }
            code.push_str(&format!("{}    w.write_all(b\"QUIT\\r\\n\").await?;\n", pad));
            code.push_str(&format!("{}    transcript\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::WebSocket(_s) => {
            code.push_str(&format!("{}// WebSocket: requires tokio-tungstenite crate\n", pad));
            code.push_str(&format!("{}// TODO: implement WebSocket connection\n", pad));
        }
        BlockSettings::CaptchaSolver(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}// Captcha solver via {} API\n", pad, escape_str(&s.solver_service)));
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    let create_resp = client.post(\"https://api.capsolver.com/createTask\")\n", pad));
            code.push_str(&format!("{}        .header(\"Content-Type\", \"application/json\")\n", pad));
            code.push_str(&format!("{}        .body(serde_json::json!({{\"clientKey\": \"{}\", \"task\": {{\"type\": \"RecaptchaV2TaskProxyless\", \"websiteURL\": \"{}\", \"websiteKey\": \"{}\"}}}}).to_string())\n",
                pad, escape_str(&s.api_key), escape_str(&s.page_url), escape_str(&s.site_key)));
            code.push_str(&format!("{}        .send().await?.text().await?;\n", pad));
            code.push_str(&format!("{}    let task_id = serde_json::from_str::<Value>(&create_resp)?[\"taskId\"].as_str().unwrap_or_default().to_string();\n", pad));
            code.push_str(&format!("{}    loop {{\n", pad));
            code.push_str(&format!("{}        tokio::time::sleep(std::time::Duration::from_secs(5)).await;\n", pad));
            code.push_str(&format!("{}        let poll = client.post(\"https://api.capsolver.com/getTaskResult\")\n", pad));
            code.push_str(&format!("{}            .header(\"Content-Type\", \"application/json\")\n", pad));
            code.push_str(&format!("{}            .body(serde_json::json!({{\"clientKey\": \"{}\", \"taskId\": task_id}}).to_string())\n", pad, escape_str(&s.api_key)));
            code.push_str(&format!("{}            .send().await?.text().await?;\n", pad));
            code.push_str(&format!("{}        let json: Value = serde_json::from_str(&poll)?;\n", pad));
            code.push_str(&format!("{}        if json[\"status\"] == \"ready\" {{ break json[\"solution\"][\"gRecaptchaResponse\"].as_str().unwrap_or_default().to_string(); }}\n", pad));
            code.push_str(&format!("{}    }}\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::CloudflareBypass(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}// Cloudflare bypass via FlareSolverr\n", pad));
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    let resp = client.post(\"{}\")\n", pad, escape_str(&s.flaresolverr_url)));
            code.push_str(&format!("{}        .header(\"Content-Type\", \"application/json\")\n", pad));
            code.push_str(&format!("{}        .body(serde_json::json!({{\"cmd\": \"request.get\", \"url\": \"{}\"}}).to_string())\n", pad, escape_str(&s.url)));
            code.push_str(&format!("{}        .send().await?.text().await?;\n", pad));
            code.push_str(&format!("{}    let json: Value = serde_json::from_str(&resp)?;\n", pad));
            code.push_str(&format!("{}    json[\"solution\"][\"cookies\"].as_array()\n", pad));
            code.push_str(&format!("{}        .map(|a| a.iter().filter_map(|c| Some(format!(\"{{}}={{}}\", c[\"name\"].as_str()?, c[\"value\"].as_str()?))).collect::<Vec<_>>().join(\"; \"))\n", pad));
            code.push_str(&format!("{}        .unwrap_or_default()\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::LaravelCsrf(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}// Laravel CSRF token extraction\n", pad));
            code.push_str(&format!("{}let csrf_page = client.get(\"{}\")\n", pad, escape_str(&s.url)));
            code.push_str(&format!("{}    .send().await?.text().await?;\n", pad));
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    let doc = Html::parse_document(&csrf_page);\n", pad));
            code.push_str(&format!("{}    let sel = Selector::parse(\"{}\").unwrap();\n", pad, escape_str(&s.csrf_selector)));
            code.push_str(&format!("{}    doc.select(&sel).next().and_then(|el| el.value().attr(\"value\")).unwrap_or_default().to_string()\n", pad));
            code.push_str(&format!("{}}};\n", pad));
        }
        // Extended functions
        BlockSettings::ByteArray(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            match s.operation {
                ByteArrayOp::ToHex => {
                    code.push_str(&format!("{}{}{}= {}.as_bytes().iter().map(|b| format!(\"{{:02x}}\", b)).collect::<String>();\n", pad, letkw, vn, input));
                }
                ByteArrayOp::FromHex => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let hex_clean: String = {}.chars().filter(|c| c.is_ascii_hexdigit()).collect();\n", pad, input));
                    code.push_str(&format!("{}    let bytes: Vec<u8> = (0..hex_clean.len()).step_by(2)\n", pad));
                    code.push_str(&format!("{}        .filter_map(|i| u8::from_str_radix(&hex_clean[i..i+2], 16).ok()).collect();\n", pad));
                    code.push_str(&format!("{}    String::from_utf8_lossy(&bytes).to_string()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                ByteArrayOp::ToBase64 => {
                    code.push_str(&format!("{}{}{}= base64::encode({}.as_bytes());\n", pad, letkw, vn, input));
                }
                ByteArrayOp::FromBase64 => {
                    code.push_str(&format!("{}{}{}= base64::decode({}.as_bytes()).ok()\n", pad, letkw, vn, input));
                    code.push_str(&format!("{}    .and_then(|b| String::from_utf8(b).ok()).unwrap_or_default();\n", pad));
                }
                ByteArrayOp::ToUtf8 => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let bytes: Vec<u8> = {}.split(',').filter_map(|s| s.trim().parse().ok()).collect();\n", pad, input));
                    code.push_str(&format!("{}    String::from_utf8_lossy(&bytes).to_string()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                ByteArrayOp::FromUtf8 => {
                    code.push_str(&format!("{}{}{}= {}.as_bytes().iter().map(|b| b.to_string()).collect::<Vec<_>>().join(\",\");\n", pad, letkw, vn, input));
                }
            }
        }
        BlockSettings::Constants(s) => {
            for constant in &s.constants {
                let vn = var_name(&constant.name);
                let letkw = vars.let_or_assign(&constant.name);
                code.push_str(&format!("{}{}{}= \"{}\".to_string();\n", pad, letkw, vn, escape_str(&constant.value)));
            }
        }
        BlockSettings::Dictionary(s) => {
            let dict_var = if vars.is_defined(&s.dict_var) { var_name(&s.dict_var) } else { "dict".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            match s.operation {
                DictOp::Get => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    serde_json::from_str::<serde_json::Value>(&{})\n", pad, dict_var));
                    code.push_str(&format!("{}        .ok().and_then(|v| v.get(\"{}\")).and_then(|v| v.as_str())\n", pad, escape_str(&s.key)));
                    code.push_str(&format!("{}        .unwrap_or(\"\").to_string()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                DictOp::Set => {
                    code.push_str(&format!("{}{}= {{\n", pad, dict_var));
                    code.push_str(&format!("{}    let mut map = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&{})\n", pad, dict_var));
                    code.push_str(&format!("{}        .unwrap_or_default();\n", pad));
                    code.push_str(&format!("{}    map.insert(\"{}\".to_string(), serde_json::Value::String(\"{}\".to_string()));\n", pad, escape_str(&s.key), escape_str(&s.value)));
                    code.push_str(&format!("{}    serde_json::to_string(&map).unwrap_or_default()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                    code.push_str(&format!("{}{}{}= {}.clone();\n", pad, letkw, vn, dict_var));
                }
                DictOp::Remove => {
                    code.push_str(&format!("{}{}= {{\n", pad, dict_var));
                    code.push_str(&format!("{}    let mut map = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&{})\n", pad, dict_var));
                    code.push_str(&format!("{}        .unwrap_or_default();\n", pad));
                    code.push_str(&format!("{}    map.remove(\"{}\");\n", pad, escape_str(&s.key)));
                    code.push_str(&format!("{}    serde_json::to_string(&map).unwrap_or_default()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                    code.push_str(&format!("{}{}{}= {}.clone();\n", pad, letkw, vn, dict_var));
                }
                DictOp::Exists => {
                    code.push_str(&format!("{}{}{}= serde_json::from_str::<serde_json::Value>(&{})\n", pad, letkw, vn, dict_var));
                    code.push_str(&format!("{}    .ok().and_then(|v| v.get(\"{}\")).is_some().to_string();\n", pad, escape_str(&s.key)));
                }
                DictOp::Keys => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&{})\n", pad, dict_var));
                    code.push_str(&format!("{}        .ok().map(|m| serde_json::to_string(&m.keys().collect::<Vec<_>>()).unwrap_or_default())\n", pad));
                    code.push_str(&format!("{}        .unwrap_or_else(|| \"[]\".to_string())\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                DictOp::Values => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&{})\n", pad, dict_var));
                    code.push_str(&format!("{}        .ok().map(|m| serde_json::to_string(&m.values().collect::<Vec<_>>()).unwrap_or_default())\n", pad));
                    code.push_str(&format!("{}        .unwrap_or_else(|| \"[]\".to_string())\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
            }
        }
        BlockSettings::FloatFunction(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            let param1 = &s.param1;
            match s.function_type {
                FloatFnType::Round => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let val: f64 = {}.parse().unwrap_or(0.0);\n", pad, input));
                    code.push_str(&format!("{}    let places: u32 = {};\n", pad, param1));
                    code.push_str(&format!("{}    let mult = 10_f64.powi(places as i32);\n", pad));
                    code.push_str(&format!("{}    ((val * mult).round() / mult).to_string()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                FloatFnType::Ceil => {
                    code.push_str(&format!("{}{}{}= {}.parse::<f64>().unwrap_or(0.0).ceil().to_string();\n", pad, letkw, vn, input));
                }
                FloatFnType::Floor => {
                    code.push_str(&format!("{}{}{}= {}.parse::<f64>().unwrap_or(0.0).floor().to_string();\n", pad, letkw, vn, input));
                }
                FloatFnType::Abs => {
                    code.push_str(&format!("{}{}{}= {}.parse::<f64>().unwrap_or(0.0).abs().to_string();\n", pad, letkw, vn, input));
                }
                FloatFnType::Add => {
                    code.push_str(&format!("{}{}{}= ({}.parse::<f64>().unwrap_or(0.0) + {}).to_string();\n", pad, letkw, vn, input, param1));
                }
                FloatFnType::Subtract => {
                    code.push_str(&format!("{}{}{}= ({}.parse::<f64>().unwrap_or(0.0) - {}).to_string();\n", pad, letkw, vn, input, param1));
                }
                FloatFnType::Multiply => {
                    code.push_str(&format!("{}{}{}= ({}.parse::<f64>().unwrap_or(0.0) * {}).to_string();\n", pad, letkw, vn, input, param1));
                }
                FloatFnType::Divide => {
                    code.push_str(&format!("{}{}{}= ({}.parse::<f64>().unwrap_or(0.0) / {}).to_string();\n", pad, letkw, vn, input, param1));
                }
                FloatFnType::Power => {
                    code.push_str(&format!("{}{}{}= {}.parse::<f64>().unwrap_or(0.0).powf({}).to_string();\n", pad, letkw, vn, input, param1));
                }
                FloatFnType::Sqrt => {
                    code.push_str(&format!("{}{}{}= {}.parse::<f64>().unwrap_or(0.0).sqrt().to_string();\n", pad, letkw, vn, input));
                }
                FloatFnType::Min => {
                    code.push_str(&format!("{}{}{}= {}.parse::<f64>().unwrap_or(0.0).min({}).to_string();\n", pad, letkw, vn, input, param1));
                }
                FloatFnType::Max => {
                    code.push_str(&format!("{}{}{}= {}.parse::<f64>().unwrap_or(0.0).max({}).to_string();\n", pad, letkw, vn, input, param1));
                }
            }
        }
        BlockSettings::IntegerFunction(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            let param1 = &s.param1;
            match s.function_type {
                IntegerFnType::Add => {
                    code.push_str(&format!("{}{}{}= ({}.parse::<i64>().unwrap_or(0) + {}).to_string();\n", pad, letkw, vn, input, param1));
                }
                IntegerFnType::Subtract => {
                    code.push_str(&format!("{}{}{}= ({}.parse::<i64>().unwrap_or(0) - {}).to_string();\n", pad, letkw, vn, input, param1));
                }
                IntegerFnType::Multiply => {
                    code.push_str(&format!("{}{}{}= ({}.parse::<i64>().unwrap_or(0) * {}).to_string();\n", pad, letkw, vn, input, param1));
                }
                IntegerFnType::Divide => {
                    code.push_str(&format!("{}{}{}= ({}.parse::<i64>().unwrap_or(0) / {}.max(1)).to_string();\n", pad, letkw, vn, input, param1));
                }
                IntegerFnType::Modulo => {
                    code.push_str(&format!("{}{}{}= ({}.parse::<i64>().unwrap_or(0) % {}.max(1)).to_string();\n", pad, letkw, vn, input, param1));
                }
                IntegerFnType::Power => {
                    code.push_str(&format!("{}{}{}= {}.parse::<i64>().unwrap_or(0).pow({} as u32).to_string();\n", pad, letkw, vn, input, param1));
                }
                IntegerFnType::Abs => {
                    code.push_str(&format!("{}{}{}= {}.parse::<i64>().unwrap_or(0).abs().to_string();\n", pad, letkw, vn, input));
                }
                IntegerFnType::Min => {
                    code.push_str(&format!("{}{}{}= {}.parse::<i64>().unwrap_or(0).min({}).to_string();\n", pad, letkw, vn, input, param1));
                }
                IntegerFnType::Max => {
                    code.push_str(&format!("{}{}{}= {}.parse::<i64>().unwrap_or(0).max({}).to_string();\n", pad, letkw, vn, input, param1));
                }
            }
        }
        BlockSettings::TimeFunction(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            match s.function_type {
                TimeFnType::ConvertTimezone => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    use chrono::{{DateTime, TimeZone}};\n", pad));
                    code.push_str(&format!("{}    use chrono_tz::Tz;\n", pad));
                    code.push_str(&format!("{}    let target_tz: Tz = \"{}\".parse().unwrap_or(chrono_tz::UTC);\n", pad, escape_str(&s.target_timezone)));
                    code.push_str(&format!("{}    if let Ok(dt) = DateTime::parse_from_rfc3339(&{}) {{\n", pad, input));
                    code.push_str(&format!("{}        dt.with_timezone(&target_tz).format(\"{}\").to_string()\n", pad, escape_str(&s.format)));
                    code.push_str(&format!("{}    }} else if let Ok(timestamp) = {}.parse::<i64>() {{\n", pad, input));
                    code.push_str(&format!("{}        let dt = chrono::Utc.timestamp_opt(timestamp, 0).unwrap();\n", pad));
                    code.push_str(&format!("{}        dt.with_timezone(&target_tz).format(\"{}\").to_string()\n", pad, escape_str(&s.format)));
                    code.push_str(&format!("{}    }} else {{ {}.to_string() }}\n", pad, input));
                    code.push_str(&format!("{}}};\n", pad));
                }
                TimeFnType::GetTimezone => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&{}) {{\n", pad, input));
                    code.push_str(&format!("{}        dt.timezone().to_string()\n", pad));
                    code.push_str(&format!("{}    }} else {{ \"UTC\".to_string() }}\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                TimeFnType::IsDST => {
                    code.push_str(&format!("{}{}{}= \"false\".to_string(); // IsDST not fully implemented\n", pad, letkw, vn));
                }
                TimeFnType::DurationBetween => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let ts1: i64 = {}.parse().unwrap_or(0);\n", pad, input));
                    code.push_str(&format!("{}    let ts2: i64 = \"{}\".parse().unwrap_or(0);\n", pad, escape_str(&s.target_timezone)));
                    code.push_str(&format!("{}    (ts2 - ts1).abs().to_string()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                TimeFnType::AddDuration => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let timestamp: i64 = {}.parse().unwrap_or(0);\n", pad, input));
                    code.push_str(&format!("{}    let duration: i64 = \"{}\".parse().unwrap_or(0);\n", pad, escape_str(&s.format)));
                    code.push_str(&format!("{}    (timestamp + duration).to_string()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                TimeFnType::SubtractDuration => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    let timestamp: i64 = {}.parse().unwrap_or(0);\n", pad, input));
                    code.push_str(&format!("{}    let duration: i64 = \"{}\".parse().unwrap_or(0);\n", pad, escape_str(&s.format)));
                    code.push_str(&format!("{}    (timestamp - duration).to_string()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
            }
        }
        BlockSettings::GenerateGUID(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            match s.guid_version {
                GUIDVersion::V1 => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    use uuid::{{Uuid, Timestamp}};\n", pad));
                    code.push_str(&format!("{}    let now = std::time::SystemTime::now();\n", pad));
                    code.push_str(&format!("{}    let ts = Timestamp::from_unix(uuid::timestamp::context::NoContext, now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(), 0);\n", pad));
                    code.push_str(&format!("{}    Uuid::new_v1(ts, &[1, 2, 3, 4, 5, 6]).to_string()\n", pad));
                    code.push_str(&format!("{}}};\n", pad));
                }
                GUIDVersion::V4 => {
                    code.push_str(&format!("{}{}{}= uuid::Uuid::new_v4().to_string();\n", pad, letkw, vn));
                }
                GUIDVersion::V5 => {
                    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
                    code.push_str(&format!("{}    use uuid::Uuid;\n", pad));
                    code.push_str(&format!("{}    let namespace = match \"{}\".to_lowercase().as_str() {{\n", pad, escape_str(&s.namespace)));
                    code.push_str(&format!("{}        \"dns\" => Uuid::NAMESPACE_DNS,\n", pad));
                    code.push_str(&format!("{}        \"url\" => Uuid::NAMESPACE_URL,\n", pad));
                    code.push_str(&format!("{}        \"oid\" => Uuid::NAMESPACE_OID,\n", pad));
                    code.push_str(&format!("{}        \"x500\" => Uuid::NAMESPACE_X500,\n", pad));
                    code.push_str(&format!("{}        _ => Uuid::parse_str(\"{}\").unwrap_or(Uuid::NAMESPACE_DNS),\n", pad, escape_str(&s.namespace)));
                    code.push_str(&format!("{}    }};\n", pad));
                    code.push_str(&format!("{}    Uuid::new_v5(&namespace, \"{}\".as_bytes()).to_string()\n", pad, escape_str(&s.name)));
                    code.push_str(&format!("{}}};\n", pad));
                }
            }
        }
        BlockSettings::PhoneCountry(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    // PhoneCountry: extract country from phone number\n", pad));
            code.push_str(&format!("{}    let country_code = {}.chars().filter(|c| c.is_numeric()).take(3).collect::<String>();\n", pad, input));
            match s.output_format {
                PhoneOutputFormat::CountryCode => {
                    code.push_str(&format!("{}    country_code\n", pad));
                }
                PhoneOutputFormat::CountryName => {
                    code.push_str(&format!("{}    match country_code.as_str() {{\n", pad));
                    code.push_str(&format!("{}        \"1\" => \"United States\",\n", pad));
                    code.push_str(&format!("{}        \"44\" => \"United Kingdom\",\n", pad));
                    code.push_str(&format!("{}        \"86\" => \"China\",\n", pad));
                    code.push_str(&format!("{}        \"91\" => \"India\",\n", pad));
                    code.push_str(&format!("{}        _ => \"Unknown\",\n", pad));
                    code.push_str(&format!("{}    }}.to_string()\n", pad));
                }
                PhoneOutputFormat::ISO2 => {
                    code.push_str(&format!("{}    match country_code.as_str() {{\n", pad));
                    code.push_str(&format!("{}        \"1\" => \"US\",\n", pad));
                    code.push_str(&format!("{}        \"44\" => \"GB\",\n", pad));
                    code.push_str(&format!("{}        \"86\" => \"CN\",\n", pad));
                    code.push_str(&format!("{}        \"91\" => \"IN\",\n", pad));
                    code.push_str(&format!("{}        _ => \"XX\",\n", pad));
                    code.push_str(&format!("{}    }}.to_string()\n", pad));
                }
                PhoneOutputFormat::ISO3 => {
                    code.push_str(&format!("{}    match country_code.as_str() {{\n", pad));
                    code.push_str(&format!("{}        \"1\" => \"USA\",\n", pad));
                    code.push_str(&format!("{}        \"44\" => \"GBR\",\n", pad));
                    code.push_str(&format!("{}        \"86\" => \"CHN\",\n", pad));
                    code.push_str(&format!("{}        \"91\" => \"IND\",\n", pad));
                    code.push_str(&format!("{}        _ => \"XXX\",\n", pad));
                    code.push_str(&format!("{}    }}.to_string()\n", pad));
                }
            }
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::LambdaParser(s) => {
            let input = if vars.is_defined(&s.input_var) { var_name(&s.input_var) } else { "source".into() };
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
            code.push_str(&format!("{}    // LambdaParser: {}\n", pad, escape_str(&s.lambda_expression)));
            code.push_str(&format!("{}    {}.to_string() // Simplified lambda execution\n", pad, input));
            code.push_str(&format!("{}}};\n", pad));
        }
        BlockSettings::DataConversion(s) => {
            let letkw = vars.let_or_assign(&s.output_var);
            let vn = var_name(&s.output_var);
            code.push_str(&format!("{}{}{}= String::new(); // DataConversion::{:?} (runtime op)\n", pad, letkw, vn, s.op));
        }
        BlockSettings::FileSystem(s) => {
            code.push_str(&format!("{}// FileSystem::{:?} on \"{}\"\n", pad, s.op, escape_str(&s.path)));
        }
    }

    code
}

// ── Script block codegen with SVB pattern recognition ──

fn generate_script_block(
    script_code: &str,
    output_var: &str,
    pad: &str,
    vars: &mut VarTracker,
    code: &mut String,
) {
    // Try to detect known SVB/OB2 patterns and generate real Rust code
    if script_code.contains("SVB Translate: lookup table") {
        generate_svb_translate(script_code, output_var, pad, vars, code);
    } else if script_code.contains("OB2 Translate block") {
        generate_ob2_translate(script_code, output_var, pad, vars, code);
    } else if script_code.contains("OB2 CountOccurrences block") {
        generate_ob2_count_occurrences(script_code, output_var, pad, vars, code);
    } else if script_code.contains("SVB UnixTimeToDate:") {
        generate_svb_unix_time(script_code, output_var, pad, vars, code);
    } else if script_code.contains("SVB Unescape:") {
        generate_svb_unescape(script_code, output_var, pad, vars, code);
    } else if script_code.contains("SVB Split:") {
        generate_svb_split(script_code, output_var, pad, vars, code);
    } else if script_code.contains("SVB UTILITY") {
        generate_svb_utility(script_code, output_var, pad, vars, code);
    } else if script_code.contains("Converted from OB2 C# preamble") {
        generate_ob2_preamble(script_code, output_var, pad, vars, code);
    } else {
        // Unknown pattern — emit as comments + TODO stub
        code.push_str(&format!("{}// Script block (original code below):\n", pad));
        for line in script_code.lines() {
            code.push_str(&format!("{}// {}\n", pad, line));
        }
        if !output_var.is_empty() {
            let letkw = vars.let_or_assign(output_var);
            code.push_str(&format!(
                "{}{}{}= String::new(); // TODO: implement script logic\n",
                pad, letkw, var_name(output_var)
            ));
        }
    }
}

/// SVB Translate: lookup table → match expression
fn generate_svb_translate(
    script_code: &str,
    output_var: &str,
    pad: &str,
    vars: &mut VarTracker,
    code: &mut String,
) {
    // Parse input variable from: "// SVB Translate: lookup table on <VAR> → OUTPUT"
    let input_var = script_code
        .lines()
        .next()
        .and_then(|l| {
            let start = l.find('<')? + 1;
            let end = l[start..].find('>')? + start;
            Some(l[start..end].to_string())
        })
        .unwrap_or_default();

    // Parse lookup entries from: "//   \"KEY\" => \"VALUE\""
    let entries: Vec<(String, String)> = script_code
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim().trim_start_matches("//").trim();
            if !trimmed.starts_with('"') || !trimmed.contains("=>") {
                return None;
            }
            let parts: Vec<&str> = trimmed.splitn(2, "=>").collect();
            if parts.len() != 2 { return None; }
            let key = parts[0].trim().trim_matches('"').to_string();
            let val = parts[1].trim().trim_matches('"').to_string();
            Some((key, val))
        })
        .collect();

    let input = if vars.is_defined(&input_var) {
        var_name(&input_var)
    } else {
        "source".into()
    };

    let out = if output_var.is_empty() { "result" } else { output_var };
    let letkw = vars.let_or_assign(out);
    let vn = var_name(out);

    code.push_str(&format!("{}{}{}= match {}.as_str() {{\n", pad, letkw, vn, input));
    for (key, val) in &entries {
        if val == "N/A" {
            code.push_str(&format!("{}    \"{}\" => \"\".to_string(),\n", pad, escape_str(key)));
        } else {
            code.push_str(&format!("{}    \"{}\" => \"{}\".to_string(),\n", pad, escape_str(key), escape_str(val)));
        }
    }
    code.push_str(&format!("{}    _ => String::new(),\n", pad));
    code.push_str(&format!("{}}};\n", pad));
}

/// SVB UnixTimeToDate: unix timestamp → formatted date string via chrono
fn generate_svb_unix_time(
    script_code: &str,
    output_var: &str,
    pad: &str,
    vars: &mut VarTracker,
    code: &mut String,
) {
    // Parse from: "// SVB UnixTimeToDate: convert <VAR> with format "FMT" → OUT (capture=BOOL)"
    let first_line = script_code.lines().next().unwrap_or("");

    let input_var = first_line
        .find('<')
        .and_then(|s| {
            let start = s + 1;
            first_line[start..].find('>').map(|e| first_line[start..start + e].to_string())
        })
        .unwrap_or_default();

    // Extract format string between the two quotes after "format"
    let format_str = first_line
        .find("format \"")
        .map(|s| {
            let start = s + 8;
            let end = first_line[start..].find('"').map(|e| start + e).unwrap_or(first_line.len());
            first_line[start..end].to_string()
        })
        .unwrap_or_else(|| "%Y-%m-%d".to_string());

    // Convert C#/Java date format tokens to chrono strftime
    let chrono_fmt = dotnet_date_to_chrono(&format_str);

    let input = if vars.is_defined(&input_var) {
        var_name(&input_var)
    } else {
        "source".into()
    };

    let out = if output_var.is_empty() { "result" } else { output_var };
    let letkw = vars.let_or_assign(out);
    let vn = var_name(out);

    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
    code.push_str(&format!("{}    let ts: i64 = {}.parse().unwrap_or(0);\n", pad, input));
    code.push_str(&format!("{}    chrono::DateTime::from_timestamp(ts, 0)\n", pad));
    code.push_str(&format!("{}        .map(|dt| dt.format(\"{}\").to_string())\n", pad, escape_str(&chrono_fmt)));
    code.push_str(&format!("{}        .unwrap_or_default()\n", pad));
    code.push_str(&format!("{}}};\n", pad));
}

/// SVB Unescape: HTML entity decoding
fn generate_svb_unescape(
    script_code: &str,
    output_var: &str,
    pad: &str,
    vars: &mut VarTracker,
    code: &mut String,
) {
    // Parse from: "// SVB Unescape: unescape <VAR>, store as OUT (capture=BOOL)"
    let first_line = script_code.lines().next().unwrap_or("");

    let input_var = first_line
        .find('<')
        .and_then(|s| {
            let start = s + 1;
            first_line[start..].find('>').map(|e| first_line[start..start + e].to_string())
        })
        .unwrap_or_default();

    let input = if vars.is_defined(&input_var) {
        var_name(&input_var)
    } else {
        "source".into()
    };

    let out = if output_var.is_empty() { "result" } else { output_var };
    let letkw = vars.let_or_assign(out);
    let vn = var_name(out);

    code.push_str(&format!("{}{}{}= {{\n", pad, letkw, vn));
    code.push_str(&format!("{}    let mut s = {}.clone();\n", pad, input));
    code.push_str(&format!("{}    s = s.replace(\"&amp;\", \"&\");\n", pad));
    code.push_str(&format!("{}    s = s.replace(\"&lt;\", \"<\");\n", pad));
    code.push_str(&format!("{}    s = s.replace(\"&gt;\", \">\");\n", pad));
    code.push_str(&format!("{}    s = s.replace(\"&quot;\", \"\\\"\");\n", pad));
    code.push_str(&format!("{}    s = s.replace(\"&#39;\", \"\\'\");\n", pad));
    code.push_str(&format!("{}    s = s.replace(\"&apos;\", \"\\'\");\n", pad));
    code.push_str(&format!("{}    s = s.replace(\"&#x2F;\", \"/\");\n", pad));
    code.push_str(&format!("{}    s = s.replace(\"&nbsp;\", \" \");\n", pad));
    // Handle numeric entities &#NNN; and &#xHHH;
    code.push_str(&format!("{}    // Decode numeric HTML entities (&#NNN; and &#xHHH;)\n", pad));
    code.push_str(&format!("{}    while let Some(start) = s.find(\"&#\") {{\n", pad));
    code.push_str(&format!("{}        if let Some(end) = s[start..].find(';') {{\n", pad));
    code.push_str(&format!("{}            let entity = &s[start + 2..start + end];\n", pad));
    code.push_str(&format!("{}            let codepoint = if entity.starts_with('x') || entity.starts_with('X') {{\n", pad));
    code.push_str(&format!("{}                u32::from_str_radix(&entity[1..], 16).ok()\n", pad));
    code.push_str(&format!("{}            }} else {{\n", pad));
    code.push_str(&format!("{}                entity.parse::<u32>().ok()\n", pad));
    code.push_str(&format!("{}            }};\n", pad));
    code.push_str(&format!("{}            if let Some(cp) = codepoint.and_then(char::from_u32) {{\n", pad));
    code.push_str(&format!("{}                s = format!(\"{{}}{{}}{{}}\", &s[..start], cp, &s[start + end + 1..]);\n", pad));
    code.push_str(&format!("{}            }} else {{ break; }}\n", pad));
    code.push_str(&format!("{}        }} else {{ break; }}\n", pad));
    code.push_str(&format!("{}    }}\n", pad));
    code.push_str(&format!("{}    s\n", pad));
    code.push_str(&format!("{}}};\n", pad));
}

/// SVB Split: split by separator, take index
fn generate_svb_split(
    script_code: &str,
    output_var: &str,
    pad: &str,
    vars: &mut VarTracker,
    code: &mut String,
) {
    // Parse from: "// SVB Split: split <VAR> by "SEP" take index N, store in OUT"
    let first_line = script_code.lines().next().unwrap_or("");

    let input_var = first_line
        .find('<')
        .and_then(|s| {
            let start = s + 1;
            first_line[start..].find('>').map(|e| first_line[start..start + e].to_string())
        })
        .unwrap_or_default();

    let separator = first_line
        .find("by \"")
        .map(|s| {
            let start = s + 4;
            let end = first_line[start..].find('"').map(|e| start + e).unwrap_or(first_line.len());
            first_line[start..end].to_string()
        })
        .unwrap_or_else(|| ",".to_string());

    let index: usize = first_line
        .find("index ")
        .and_then(|s| {
            let start = s + 6;
            first_line[start..].split(|c: char| !c.is_ascii_digit()).next()
                .and_then(|n| n.parse().ok())
        })
        .unwrap_or(0);

    let input = if vars.is_defined(&input_var) {
        var_name(&input_var)
    } else {
        "source".into()
    };

    let out = if output_var.is_empty() { "result" } else { output_var };
    let letkw = vars.let_or_assign(out);
    let vn = var_name(out);

    code.push_str(&format!(
        "{}{}{}= {}.split(\"{}\").nth({}).unwrap_or(\"\").to_string();\n",
        pad, letkw, vn, input, escape_str(&separator), index
    ));
}

/// SVB UTILITY: file operations and other utilities
fn generate_svb_utility(
    script_code: &str,
    output_var: &str,
    pad: &str,
    vars: &mut VarTracker,
    code: &mut String,
) {
    // Extract the actual utility command line (second line, after "// SVB UTILITY — ...")
    let cmd_line = script_code
        .lines()
        .nth(1)
        .map(|l| l.trim().trim_start_matches("//").trim())
        .unwrap_or("");

    // UTILITY File "path" AppendLines "content" -> VAR "name"
    if cmd_line.starts_with("UTILITY File ") {
        let rest = &cmd_line[13..];
        // Extract file path
        let (file_path, rest) = extract_quoted_from_codegen(rest);
        let rest = rest.trim();

        if rest.starts_with("AppendLines ") {
            let rest = &rest[12..];
            let (content_template, _rest) = extract_quoted_from_codegen(rest);

            // Convert <VAR> interpolation to Rust format!() args
            let (fmt_str, fmt_args) = convert_interpolation_to_format(&content_template);

            code.push_str(&format!("{}// Append to file: {}\n", pad, escape_str(&file_path)));
            code.push_str(&format!("{}{{\n", pad));
            code.push_str(&format!("{}    let path = std::path::Path::new(\"{}\");\n", pad, escape_str(&file_path)));
            code.push_str(&format!("{}    if let Some(parent) = path.parent() {{\n", pad));
            code.push_str(&format!("{}        std::fs::create_dir_all(parent).ok();\n", pad));
            code.push_str(&format!("{}    }}\n", pad));
            code.push_str(&format!("{}    use std::io::Write;\n", pad));
            code.push_str(&format!("{}    let mut f = std::fs::OpenOptions::new()\n", pad));
            code.push_str(&format!("{}        .create(true).append(true).open(path)?;\n", pad));
            if fmt_args.is_empty() {
                code.push_str(&format!("{}    writeln!(f, \"{}\")?;\n", pad, escape_str(&fmt_str)));
            } else {
                code.push_str(&format!("{}    writeln!(f, \"{}\", {})?;\n", pad, escape_str(&fmt_str), fmt_args));
            }
            code.push_str(&format!("{}}}\n", pad));

            if !output_var.is_empty() {
                let letkw = vars.let_or_assign(output_var);
                code.push_str(&format!("{}{}{}= \"ok\".to_string();\n", pad, letkw, var_name(output_var)));
            }
            return;
        }
    }

    // Fallback: emit as comment
    code.push_str(&format!("{}// SVB UTILITY (unrecognized pattern):\n", pad));
    for line in script_code.lines() {
        code.push_str(&format!("{}// {}\n", pad, line));
    }
    if !output_var.is_empty() {
        let letkw = vars.let_or_assign(output_var);
        code.push_str(&format!("{}{}{}= String::new(); // TODO: implement utility\n", pad, letkw, var_name(output_var)));
    }
}

/// OB2 Translate: lookup table with tuple-style entries → match expression
fn generate_ob2_translate(
    script_code: &str,
    output_var: &str,
    pad: &str,
    vars: &mut VarTracker,
    code: &mut String,
) {
    // Parse input variable from: "// input = @varname"
    let input_var = parse_ob2_line_var(script_code, "input = @");

    // Parse translations from: // translations = {("KEY", "VALUE"), ("KEY2", "VALUE2"), ...}
    let entries: Vec<(String, String)> = script_code
        .lines()
        .find(|l| l.contains("translations = {"))
        .map(|line| {
            let trimmed = line.trim().trim_start_matches("//").trim();
            let start = trimmed.find('{').unwrap_or(0) + 1;
            let end = trimmed.rfind('}').unwrap_or(trimmed.len());
            let inner = &trimmed[start..end];
            let mut result = Vec::new();
            let mut rest = inner;
            while let Some(paren_start) = rest.find('(') {
                let after = &rest[paren_start + 1..];
                if let Some(paren_end) = after.find(')') {
                    let tuple_inner = &after[..paren_end];
                    let parts: Vec<&str> = tuple_inner.splitn(2, ", ").collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim().trim_matches('"').to_string();
                        let val = parts[1].trim().trim_matches('"').to_string();
                        result.push((key, val));
                    }
                    rest = &after[paren_end + 1..];
                } else {
                    break;
                }
            }
            result
        })
        .unwrap_or_default();

    let input = if vars.is_defined(&input_var) {
        var_name(&input_var)
    } else {
        "source".into()
    };

    // Use output_var from settings, or fall back to comment-parsed output
    let out = resolve_ob2_output_var(output_var, script_code);
    let letkw = vars.let_or_assign(&out);
    let vn = var_name(&out);

    code.push_str(&format!("{}{}{}= match {}.as_str() {{\n", pad, letkw, vn, input));
    for (key, val) in &entries {
        code.push_str(&format!("{}    \"{}\" => \"{}\".to_string(),\n", pad, escape_str(key), escape_str(val)));
    }
    code.push_str(&format!("{}    _ => {}.to_string(),\n", pad, input));
    code.push_str(&format!("{}}};\n", pad));
}

/// OB2 CountOccurrences: count how many times a word appears in input
fn generate_ob2_count_occurrences(
    script_code: &str,
    output_var: &str,
    pad: &str,
    vars: &mut VarTracker,
    code: &mut String,
) {
    let input_var = parse_ob2_line_var(script_code, "input = @");

    // Parse word from: // word = "text"
    let word = script_code
        .lines()
        .find_map(|l| {
            let t = l.trim().trim_start_matches("//").trim();
            if let Some(rest) = t.strip_prefix("word = ") {
                Some(rest.trim().trim_matches('"').to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let input = if vars.is_defined(&input_var) {
        var_name(&input_var)
    } else {
        "source".into()
    };

    let out = resolve_ob2_output_var(output_var, script_code);
    let letkw = vars.let_or_assign(&out);
    let vn = var_name(&out);

    code.push_str(&format!(
        "{}{}{}= {}.matches(\"{}\").count().to_string();\n",
        pad, letkw, vn, input, escape_str(&word)
    ));
}

/// Parse "// prefix@varname" from OB2 script code comments
fn parse_ob2_line_var(script_code: &str, prefix: &str) -> String {
    script_code
        .lines()
        .find_map(|l| {
            let t = l.trim().trim_start_matches("//").trim();
            t.strip_prefix(prefix).map(|v| v.trim().to_string())
        })
        .unwrap_or_default()
}

/// Resolve the actual output variable: use settings output_var unless it's the
/// default "RESULT", in which case parse from "=> CAP @Var" / "=> VAR @Var" comments.
fn resolve_ob2_output_var<'a>(output_var: &'a str, script_code: &str) -> String {
    if !output_var.is_empty() && output_var != "RESULT" {
        return output_var.to_string();
    }
    // Fall back to parsing from comment: "// Output: => CAP @VarName" or "// => CAP @VarName"
    for line in script_code.lines() {
        let t = line.trim().trim_start_matches("//").trim();
        if let Some(rest) = t.strip_prefix("=> CAP @")
            .or_else(|| t.strip_prefix("=> VAR @"))
        {
            let var = rest.trim();
            if !var.is_empty() {
                return var.to_string();
            }
        }
        // Also handle "Output: => CAP @VarName"
        if let Some(rest) = t.strip_prefix("Output: => CAP @")
            .or_else(|| t.strip_prefix("Output: => VAR @"))
        {
            let var = rest.trim();
            if !var.is_empty() {
                return var.to_string();
            }
        }
    }
    output_var.to_string()
}

/// OB2 C# Preamble: proxy setup, GUID, ConstantString, MatchRegexGroups
fn generate_ob2_preamble(
    script_code: &str,
    _output_var: &str,
    pad: &str,
    vars: &mut VarTracker,
    code: &mut String,
) {
    // Strip "// " comment prefixes from each C# line
    let raw_lines: Vec<String> = script_code
        .lines()
        .map(|l| l.trim().trim_start_matches("//").trim().to_string())
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with("Converted from OB2")
                && !l.contains("data.LogVariableAssignment")
                && !l.starts_with("data.UseProxy =")
        })
        .collect();

    let has_proxy_setup = raw_lines
        .iter()
        .any(|l| l.contains("data.UseProxy") || l.contains("data.Proxy"));
    let has_guid = raw_lines.iter().any(|l| l.contains("Guid.NewGuid()"));

    // Proxy setup — the preamble reformats OB2 proxy (type:host:port:user:pass)
    // into http://user:pass@host:port. In standalone code, proxy is configured
    // on the client builder, so we emit a helper that reads from env.
    if has_proxy_setup {
        code.push_str(&format!(
            "{}// OB2 proxy setup — reads HTTP_PROXY env var or pass as argument\n",
            pad
        ));
        let letkw = vars.let_or_assign("proxy");
        code.push_str(&format!(
            "{}{}proxy = std::env::var(\"HTTP_PROXY\").unwrap_or_default();\n",
            pad, letkw
        ));
    }

    // GUID generation
    if has_guid {
        let letkw = vars.let_or_assign("guid");
        code.push_str(&format!(
            "{}{}guid = uuid::Uuid::new_v4().to_string();\n",
            pad, letkw
        ));
    }

    // Process remaining lines: translate ConstantString and MatchRegexGroups
    // that are NOT inside the proxy if-block
    let mut in_proxy_block = false;
    let mut brace_depth: i32 = 0;

    for line in &raw_lines {
        // Skip braces
        if line.trim() == "{" {
            if in_proxy_block {
                brace_depth += 1;
            }
            continue;
        }
        if line.trim() == "}" {
            if in_proxy_block {
                brace_depth -= 1;
                if brace_depth <= 0 {
                    in_proxy_block = false;
                }
            }
            continue;
        }

        // Detect proxy if-block start
        if line.contains("CheckCondition") && line.contains("UseProxy") {
            in_proxy_block = true;
            brace_depth = 0;
            continue;
        }

        // Skip everything inside proxy block (already handled above)
        if in_proxy_block {
            continue;
        }

        // Skip lines we already handled
        if line.contains("Guid.NewGuid()") {
            continue;
        }
        // Skip proxy-related ConstantString("") init
        if line.contains("ConstantString(data,") && line.contains("proxy") {
            continue;
        }

        // ConstantString(data, "literal") or ConstantString(data, $"interpolated")
        if line.contains("ConstantString(data,") {
            if let Some((var_name_str, value)) = parse_csharp_constant_string(line) {
                let vn = var_name(&var_name_str);
                let letkw = vars.let_or_assign(&var_name_str);
                if value.starts_with("$\"") {
                    // C# interpolated string: $"prefix{var}suffix"
                    let inner = &value[2..value.len().saturating_sub(1)];
                    let (fmt, args) = convert_csharp_interpolation(inner);
                    if args.is_empty() {
                        code.push_str(&format!(
                            "{}{}{}= \"{}\".to_string();\n",
                            pad, letkw, vn, escape_str(&fmt)
                        ));
                    } else {
                        code.push_str(&format!(
                            "{}{}{}= format!(\"{}\", {});\n",
                            pad, letkw, vn, escape_str(&fmt), args
                        ));
                    }
                } else {
                    let lit = value.trim_matches('"');
                    code.push_str(&format!(
                        "{}{}{}= \"{}\".to_string();\n",
                        pad, letkw, vn, escape_str(lit)
                    ));
                }
                continue;
            }
        }

        // MatchRegexGroups(data, input.AsString(), "pattern", "[N]", ...)
        if line.contains("MatchRegexGroups(data,") {
            if let Some((var_name_str, input, pattern, group)) =
                parse_csharp_match_regex(line)
            {
                let vn = var_name(&var_name_str);
                let input_vn = var_name(&input);
                let letkw = vars.let_or_assign(&var_name_str);
                code.push_str(&format!(
                    "{}{}{}= Regex::new(r\"{}\")\n",
                    pad, letkw, vn, pattern
                ));
                code.push_str(&format!(
                    "{}    .ok().and_then(|re| re.captures(&{}))\n",
                    pad, input_vn
                ));
                code.push_str(&format!(
                    "{}    .and_then(|c| c.get({}).map(|m| m.as_str().to_string()))\n",
                    pad, group
                ));
                code.push_str(&format!("{}    .unwrap_or_default();\n", pad));
                continue;
            }
        }

        // Anything else unrecognized — emit as comment (skip noise)
        if !line.contains("data.Log") && line.trim() != "{" && line.trim() != "}" {
            code.push_str(&format!("{}// C#: {}\n", pad, line));
        }
    }
}

/// Parse C# `string VAR = ConstantString(data, "VALUE");` → (var_name, value)
fn parse_csharp_constant_string(line: &str) -> Option<(String, String)> {
    // Pattern: (string|var)? VAR = ConstantString(data, VALUE);
    let assign_pos = line.find('=')?;
    let lhs = line[..assign_pos].trim();
    let var_name_str = lhs
        .strip_prefix("string ")
        .or_else(|| lhs.strip_prefix("var "))
        .unwrap_or(lhs)
        .trim()
        .to_string();

    let rhs = line[assign_pos + 1..].trim();
    let inner_start = rhs.find("ConstantString(data,")? + 20;
    let inner = rhs[inner_start..].trim().trim_end_matches(';').trim_end_matches(')').trim();

    Some((var_name_str, inner.to_string()))
}

/// Parse C# `string VAR = MatchRegexGroups(data, INPUT.AsString(), "PATTERN", "[N]", ...);`
/// → (var_name, input_var, pattern, group_index)
fn parse_csharp_match_regex(line: &str) -> Option<(String, String, String, usize)> {
    let assign_pos = line.find('=')?;
    let lhs = line[..assign_pos].trim();
    let var_name_str = lhs
        .strip_prefix("string ")
        .or_else(|| lhs.strip_prefix("var "))
        .unwrap_or(lhs)
        .trim()
        .to_string();

    let rhs = line[assign_pos + 1..].trim();
    let inner_start = rhs.find("MatchRegexGroups(data,")? + 22;
    let inner = rhs[inner_start..].trim();

    // Split by comma — args: INPUT.AsString(), "PATTERN", "[N]", ...
    // First arg: input variable (may have .AsString())
    let args: Vec<&str> = split_csharp_args(inner);
    if args.len() < 3 {
        return None;
    }

    let input = args[0]
        .trim()
        .trim_end_matches(".AsString()")
        .trim_end_matches(".ToString()")
        .to_string();

    let pattern = args[1].trim().trim_matches('"').to_string();

    let group_str = args[2].trim().trim_matches('"');
    let group: usize = group_str
        .trim_start_matches('[')
        .trim_end_matches(']')
        .parse()
        .unwrap_or(1);

    Some((var_name_str, input, pattern, group))
}

/// Split C# function arguments respecting quotes and nested parens
fn split_csharp_args(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut in_quote = false;
    let mut start = 0;
    let bytes = s.as_bytes();

    for i in 0..bytes.len() {
        match bytes[i] {
            b'"' if depth == 0 => in_quote = !in_quote,
            b'(' if !in_quote => depth += 1,
            b')' if !in_quote => {
                if depth == 0 {
                    // End of function call
                    result.push(&s[start..i]);
                    return result;
                }
                depth -= 1;
            }
            b',' if !in_quote && depth == 0 => {
                result.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        result.push(s[start..].trim_end_matches(';').trim_end_matches(')'));
    }
    result
}

/// Convert C# string interpolation `{var}` to Rust `format!("{}", var)` style
fn convert_csharp_interpolation(template: &str) -> (String, String) {
    let mut fmt = String::new();
    let mut args = Vec::new();
    let mut rest = template;

    while let Some(start) = rest.find('{') {
        fmt.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        if let Some(end) = after.find('}') {
            let expr = after[..end].trim();
            fmt.push_str("{}");
            // Convert C# expressions: data.Proxy?.ToString() → proxy, etc.
            let arg = expr
                .replace("data.Proxy?.ToString()", "proxy")
                .replace("data.Proxy", "proxy")
                .replace("?.ToString()", "")
                .replace(".ToString()", "");
            args.push(var_name(&arg));
            rest = &after[end + 1..];
        } else {
            fmt.push('{');
            rest = after;
        }
    }
    fmt.push_str(rest);

    (fmt, args.join(", "))
}

// ── Script codegen helpers ──

/// Extract a quoted string from codegen text (handles escaped quotes)
fn extract_quoted_from_codegen(s: &str) -> (String, &str) {
    let s = s.trim();
    if !s.starts_with('"') {
        // Try to find first space as delimiter for unquoted value
        let end = s.find(' ').unwrap_or(s.len());
        return (s[..end].to_string(), &s[end..]);
    }
    let bytes = s.as_bytes();
    let mut i = 1;
    let mut result = String::new();
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'n' => result.push('\n'),
                b't' => result.push('\t'),
                b'"' => result.push('"'),
                b'\\' => result.push('\\'),
                other => { result.push('\\'); result.push(other as char); }
            }
            i += 2;
        } else if bytes[i] == b'"' {
            return (result, &s[i + 1..]);
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    (result, "")
}

/// Convert <VAR> interpolation syntax to Rust format!() style.
/// Returns (format_string, comma_separated_args).
fn convert_interpolation_to_format(template: &str) -> (String, String) {
    let mut fmt = String::new();
    let mut args = Vec::new();
    let mut rest = template;

    while let Some(start) = rest.find('<') {
        fmt.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        if let Some(end) = after.find('>') {
            let var = &after[..end];
            fmt.push_str("{}");
            args.push(var_name(var));
            rest = &after[end + 1..];
        } else {
            fmt.push('<');
            rest = after;
        }
    }
    fmt.push_str(rest);

    (fmt, args.join(", "))
}

/// Convert .NET/Java date format tokens to chrono strftime equivalents
fn dotnet_date_to_chrono(fmt: &str) -> String {
    fmt.replace("yyyy", "%Y")
        .replace("yy", "%y")
        .replace("MMMM", "%B")
        .replace("MMM", "%b")
        .replace("MM", "%m")
        .replace("dd", "%d")
        .replace("HH", "%H")
        .replace("hh", "%I")
        .replace("mm", "%M")
        .replace("ss", "%S")
        .replace("tt", "%p")
        .replace("fff", "%3f")
        .replace("ff", "%f")
}
