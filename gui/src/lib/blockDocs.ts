export interface BlockDoc {
	type: string;
	name: string;
	category: string;
	description: string;
	parameters: Array<{
		name: string;
		type: string;
		required: boolean;
		description: string;
		default?: string;
	}>;
	codeExample: string;
	tips: string[];
	relatedBlocks: string[];
	rustCode?: string;
}

export interface GuideSection {
	id: string;
	title: string;
	icon: string;
	content: string;
}

export const BLOCK_DOCS_FULL: BlockDoc[] = [
	{
		type: 'HttpRequest',
		name: 'HTTP Request',
		category: 'Requests',
		description: 'Sends an HTTP request to a URL and stores the response in variables. Supports GET, POST, PUT, DELETE, PATCH with custom headers, body, and cookies.',
		parameters: [
			{ name: 'method', type: 'string', required: true, description: 'HTTP method (GET, POST, PUT, DELETE, PATCH, HEAD)', default: 'GET' },
			{ name: 'url', type: 'string', required: true, description: 'Target URL. Supports variable interpolation with <VAR>' },
			{ name: 'headers', type: 'array', required: false, description: 'Custom HTTP headers as key-value pairs' },
			{ name: 'body', type: 'string', required: false, description: 'Request body content' },
			{ name: 'body_type', type: 'enum', required: false, description: 'Body encoding: None, Standard, Raw, Multipart, BasicAuth', default: 'None' },
			{ name: 'content_type', type: 'string', required: false, description: 'Content-Type header value', default: 'application/x-www-form-urlencoded' },
			{ name: 'follow_redirects', type: 'boolean', required: false, description: 'Follow HTTP redirects automatically', default: 'true' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Request timeout in milliseconds', default: '10000' },
			{ name: 'response_var', type: 'string', required: false, description: 'Variable prefix for response storage', default: 'SOURCE' },
		],
		codeExample: `URL: https://example.com/api/login
Method: POST
Body: username=<USER>&password=<PASS>
→ Stores response in data.SOURCE, status in data.RESPONSECODE`,
		tips: [
			'Response body is stored in data.{response_var}, headers in data.{response_var}.HEADERS',
			'Status code is always available as data.RESPONSECODE',
			'Use <USER> and <PASS> to inject credentials from the wordlist',
			'Custom cookies are sent in addition to the cookie jar',
		],
		relatedBlocks: ['ParseLR', 'ParseRegex', 'ParseJSON', 'KeyCheck'],
		rustCode: `// Variable interpolation on all string fields
let url = self.variables.interpolate(&settings.url);
let body = self.variables.interpolate(&settings.body);
let headers: Vec<(String, String)> = settings.headers.iter()
    .map(|(k, v)| (self.variables.interpolate(k), self.variables.interpolate(v)))
    .collect();

// Send via sidecar (TLS fingerprint-aware HTTP client)
let req = SidecarRequest {
    method: settings.method.clone(),
    url: url.clone(),
    headers, body,
    proxy: self.proxy.clone(),
    ja3: self.override_ja3.clone(),
    follow_redirects: settings.follow_redirects,
    timeout: settings.timeout_ms,
};
let resp = sidecar_tx.send(req).await?;

// Store response in variable namespace
let pfx = &settings.response_var; // default: "SOURCE"
self.variables.set_data(pfx, resp.body);
self.variables.set_data(&format!("{pfx}.STATUS"), resp.status.to_string());
self.variables.set_data(&format!("{pfx}.HEADERS"), serde_json::to_string(&resp.headers)?);
self.variables.set_data(&format!("{pfx}.COOKIES"), serde_json::to_string(&resp.cookies)?);
self.variables.set_data("RESPONSECODE", resp.status.to_string());`,
	},
	{
		type: 'ParseLR',
		name: 'Parse LR',
		category: 'Parsing',
		description: 'Extracts text between a left and right delimiter from a source string. Supports recursive extraction for multiple matches.',
		parameters: [
			{ name: 'input_var', type: 'string', required: true, description: 'Variable containing the text to parse', default: 'data.SOURCE' },
			{ name: 'left', type: 'string', required: true, description: 'Left boundary string' },
			{ name: 'right', type: 'string', required: true, description: 'Right boundary string' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'PARSED' },
			{ name: 'recursive', type: 'boolean', required: false, description: 'Extract all matches (creates a list)', default: 'false' },
			{ name: 'capture', type: 'boolean', required: false, description: 'Mark as captured output variable', default: 'false' },
		],
		codeExample: `Input: data.SOURCE
Left: "token":"
Right: "
→ Extracts the token value between quotes`,
		tips: [
			'Use recursive mode to find all occurrences and store them as a list',
			'Case insensitive mode ignores letter casing in delimiters',
			'For JSON data, prefer ParseJSON for reliable extraction',
		],
		relatedBlocks: ['ParseRegex', 'ParseJSON', 'ParseCSS'],
		rustCode: `let source = self.variables.get(&settings.input_var);
let left = self.variables.interpolate(&settings.left);
let right = self.variables.interpolate(&settings.right);

let mut results = Vec::new();
let mut search_from = 0;
while search_from < source.len() {
    let l_idx = match source[search_from..].find(&left) {
        Some(i) => search_from + i,
        None => break,
    };
    let start = l_idx + left.len();
    let r_idx = match source[start..].find(&right) {
        Some(i) => start + i,
        None => break,
    };
    results.push(source[start..r_idx].to_string());
    search_from = r_idx + right.len();
    if !settings.recursive { break; }
}
// Store result (single value or list)
self.variables.set(&settings.output_var, results);`,
	},
	{
		type: 'ParseRegex',
		name: 'Parse Regex',
		category: 'Parsing',
		description: 'Extracts text using a regular expression pattern with capture groups. The output format specifies how to combine captured groups.',
		parameters: [
			{ name: 'input_var', type: 'string', required: true, description: 'Variable containing the text to parse', default: 'data.SOURCE' },
			{ name: 'pattern', type: 'string', required: true, description: 'Regular expression pattern with capture groups' },
			{ name: 'output_format', type: 'string', required: false, description: 'Output template using $1, $2 etc. for groups', default: '$1' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'PARSED' },
			{ name: 'multi_line', type: 'boolean', required: false, description: 'Enable multi-line regex mode', default: 'false' },
		],
		codeExample: `Pattern: "email":"([^"]+)"
Output format: $1
→ Captures the email address from JSON`,
		tips: [
			'Use $1, $2 in output format to reference capture groups',
			'Multi-line mode makes ^ and $ match line boundaries',
			'Escape special regex chars: . * + ? [ ] ( ) { } \\ ^ $ |',
		],
		relatedBlocks: ['ParseLR', 'ParseJSON', 'ParseCSS'],
		rustCode: `let source = self.variables.get(&settings.input_var);
let pattern = self.variables.interpolate(&settings.pattern);
let re = Regex::new(&pattern)?;

let mut results = Vec::new();
for cap in re.captures_iter(&source) {
    // Build output from format string: "$1 $2" etc.
    let mut output = settings.output_format.clone();
    for i in 1..cap.len() {
        let group = cap.get(i).map(|m| m.as_str()).unwrap_or("");
        output = output.replace(&format!("\${i}"), group);
    }
    results.push(output);
    if !settings.recursive { break; }
}
self.variables.set(&settings.output_var, results);`,
	},
	{
		type: 'ParseJSON',
		name: 'Parse JSON',
		category: 'Parsing',
		description: 'Extracts a value from a JSON response using a JSONPath expression. Ideal for API responses.',
		parameters: [
			{ name: 'input_var', type: 'string', required: true, description: 'Variable containing JSON text', default: 'data.SOURCE' },
			{ name: 'json_path', type: 'string', required: true, description: 'JSONPath expression (e.g. $.data.token)' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'PARSED' },
		],
		codeExample: `Input: data.SOURCE
JSONPath: $.access_token
→ Extracts the access_token field from JSON response`,
		tips: [
			'Use $.field for top-level fields, $.nested.field for nested values',
			'Array access: $.items[0].name for first element',
			'Wildcard: $.items[*].id extracts all IDs into a list',
		],
		relatedBlocks: ['ParseLR', 'ParseRegex', 'HttpRequest'],
		rustCode: `let source = self.variables.get(&settings.input_var).unwrap_or_default();
let path = self.variables.interpolate(&settings.json_path);
let json: serde_json::Value = serde_json::from_str(&source)?;
// Supports dot notation: "user.name" → "/user/name"
let pointer = format!("/{}", path.replace('.', "/"));
let value = json.pointer(&pointer)
    .map(|v| match v { Value::String(s) => s.clone(), other => other.to_string() })
    .unwrap_or_default();
self.variables.set_user(&settings.output_var, value, settings.capture);`,
	},
	{
		type: 'ParseCSS',
		name: 'Parse CSS',
		category: 'Parsing',
		description: 'Extracts content from HTML using CSS selectors. Can read inner text or specific attributes.',
		parameters: [
			{ name: 'input_var', type: 'string', required: true, description: 'Variable containing HTML text', default: 'data.SOURCE' },
			{ name: 'selector', type: 'string', required: true, description: 'CSS selector (e.g. div.class, #id, input[name])' },
			{ name: 'attribute', type: 'string', required: false, description: 'HTML attribute to extract. Empty = innerText', default: 'innerText' },
			{ name: 'index', type: 'number', required: false, description: 'Which match to return (0 = first)', default: '0' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'PARSED' },
		],
		codeExample: `Selector: input[name="csrf_token"]
Attribute: value
→ Extracts the CSRF token from a hidden form field`,
		tips: [
			'Use "innerText" attribute to get visible text content',
			'For form tokens, target input[name="..."] with attribute "value"',
			'Index -1 returns the last matching element',
		],
		relatedBlocks: ['ParseXPath', 'ParseLR', 'ParseRegex'],
		rustCode: `let source = self.variables.get(&settings.input_var).unwrap_or_default();
let document = scraper::Html::parse_document(&source);
let selector = scraper::Selector::parse(&settings.selector)?;
let elements: Vec<_> = document.select(&selector).collect();
let el = &elements[settings.index as usize];
let value = if settings.attribute.is_empty() || settings.attribute == "text" {
    el.text().collect::<Vec<_>>().join("")
} else if settings.attribute == "innerHTML" {
    el.inner_html()
} else {
    el.value().attr(&settings.attribute).unwrap_or("").to_string()
};
self.variables.set_user(&settings.output_var, value.trim().to_string(), settings.capture);`,
	},
	{
		type: 'ParseXPath',
		name: 'Parse XPath',
		category: 'Parsing',
		description: 'Extracts content from HTML/XML using XPath expressions. More powerful than CSS selectors for complex queries.',
		parameters: [
			{ name: 'input_var', type: 'string', required: true, description: 'Variable containing HTML/XML text', default: 'data.SOURCE' },
			{ name: 'xpath', type: 'string', required: true, description: 'XPath expression' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'PARSED' },
		],
		codeExample: `XPath: //input[@name='token']/@value
→ Extracts the token value attribute`,
		tips: [
			'Use // for recursive search, / for direct children',
			'@attribute to select attributes, text() for text content',
			'Prefer CSS selectors for simpler queries',
		],
		relatedBlocks: ['ParseCSS', 'ParseLR', 'ParseRegex'],
		rustCode: `let source = self.variables.get(&settings.input_var).unwrap_or_default();
let xpath_str = self.variables.interpolate(&settings.xpath);
let package = sxd_document::parser::parse(&source)?;
let doc = package.as_document();
let xpath = sxd_xpath::Factory::new().build(&xpath_str)?.unwrap();
let result = xpath.evaluate(&sxd_xpath::Context::new(), doc.root())?;
let value = match result {
    Value::String(s) => s,
    Value::Nodeset(ns) => ns.iter().map(|n| n.string_value()).collect::<Vec<_>>().join(", "),
    _ => String::new(),
};
self.variables.set_user(&settings.output_var, value, settings.capture);`,
	},
	{
		type: 'ParseCookie',
		name: 'Parse Cookie',
		category: 'Parsing',
		description: 'Extracts a specific cookie value from the cookie jar by name.',
		parameters: [
			{ name: 'input_var', type: 'string', required: true, description: 'Variable containing cookies', default: 'data.COOKIES' },
			{ name: 'cookie_name', type: 'string', required: true, description: 'Name of the cookie to extract' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the cookie value', default: 'PARSED' },
		],
		codeExample: `Cookie name: session_id
→ Extracts the session_id cookie value`,
		tips: [
			'Cookies are automatically collected from HTTP responses',
			'Use ClearCookies block to reset the cookie jar between requests',
		],
		relatedBlocks: ['HttpRequest', 'ClearCookies', 'CookieContainer'],
		rustCode: `let source = self.variables.get(&settings.input_var).unwrap_or_default();
let cookie_name = self.variables.interpolate(&settings.cookie_name);
// Try JSON object first, then "name=value; ..." header format
let value = if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&source) {
    map.get(&cookie_name).cloned().unwrap_or_default()
} else {
    source.split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == cookie_name)
        .map(|(_, v)| v.to_string())
        .unwrap_or_default()
};
self.variables.set_user(&settings.output_var, value, settings.capture);`,
	},
	{
		type: 'KeyCheck',
		name: 'Key Check',
		category: 'Checks',
		description: 'Evaluates conditions against variables and sets the bot status (Success, Fail, Ban, Retry, Custom). Multiple keychains are checked in order.',
		parameters: [
			{ name: 'keychains', type: 'array', required: true, description: 'List of keychains, each with a result status and conditions' },
		],
		codeExample: `Keychain 1: SUCCESS when data.RESPONSECODE EqualTo "200"
Keychain 2: BAN when data.RESPONSECODE EqualTo "403"
Keychain 3: FAIL (default)`,
		tips: [
			'Keychains are evaluated top-to-bottom; first match wins',
			'Use Contains for partial string matching, EqualTo for exact match',
			'Exists/NotExists checks if a variable is set (non-empty)',
			'GreaterThan/LessThan compare numeric values',
		],
		relatedBlocks: ['HttpRequest', 'IfElse', 'CaseSwitch'],
		rustCode: `for keychain in &settings.keychains {
    let all_match = keychain.conditions.iter().all(|cond| {
        let left = self.variables.get(&cond.source);
        match cond.comparison {
            EqualTo => left == cond.value,
            Contains => left.contains(&cond.value),
            GreaterThan => left.parse::<f64>() > cond.value.parse::<f64>(),
            LessThan => left.parse::<f64>() < cond.value.parse::<f64>(),
            Exists => !left.is_empty(),
            NotExists => left.is_empty(),
            MatchesRegex => Regex::new(&cond.value).map(|r| r.is_match(&left)).unwrap_or(false),
            // ... more comparisons
        }
    });
    if all_match {
        self.bot_status = keychain.result.clone(); // SUCCESS, FAIL, BAN, etc.
        break;
    }
}`,
	},
	{
		type: 'StringFunction',
		name: 'String Function',
		category: 'Functions',
		description: 'Performs string operations: replace, substring, trim, case conversion, encoding/decoding, split, reverse, and more.',
		parameters: [
			{ name: 'function_type', type: 'enum', required: true, description: 'String operation: Replace, Substring, Trim, ToUpper, ToLower, URLEncode, URLDecode, Base64Encode, Base64Decode, Split, Reverse, Length' },
			{ name: 'input_var', type: 'string', required: true, description: 'Variable containing the input string' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'RESULT' },
			{ name: 'param1', type: 'string', required: false, description: 'First parameter (meaning depends on function type)' },
			{ name: 'param2', type: 'string', required: false, description: 'Second parameter (meaning depends on function type)' },
		],
		codeExample: `Function: Replace
Input: data.SOURCE
Param1: old_text (find)
Param2: new_text (replace with)`,
		tips: [
			'Replace: param1 = search, param2 = replacement',
			'Substring: param1 = start index, param2 = length',
			'Split: param1 = delimiter, creates a list variable',
			'URLEncode is essential for form data with special characters',
		],
		relatedBlocks: ['ConversionFunction', 'CryptoFunction', 'ListFunction'],
		rustCode: `let input = self.variables.get(&settings.input_var).unwrap_or_default();
let param1 = self.variables.interpolate(&settings.param1);
let param2 = self.variables.interpolate(&settings.param2);

let result = match settings.function_type {
    Replace => input.replace(&param1, &param2),
    Substring => {
        let start: usize = param1.parse().unwrap_or(0);
        let len: usize = param2.parse().unwrap_or(input.len());
        input.chars().skip(start).take(len).collect()
    }
    Trim => input.trim().to_string(),
    ToUpper => input.to_uppercase(),
    ToLower => input.to_lowercase(),
    URLEncode => urlencoding(&input),
    Base64Encode => base64::STANDARD.encode(input.as_bytes()),
    Split => serde_json::to_string(
        &input.split(&param1).collect::<Vec<_>>()
    ).unwrap_or_default(),
    Reverse => input.chars().rev().collect(),
    Length => input.len().to_string(),
    // ...
};
self.variables.set_user(&settings.output_var, result, settings.capture);`,
	},
	{
		type: 'ListFunction',
		name: 'List Function',
		category: 'Functions',
		description: 'Performs operations on list variables: join, sort, shuffle, add, remove, deduplicate, random item, length.',
		parameters: [
			{ name: 'function_type', type: 'enum', required: true, description: 'List operation: Join, Sort, Shuffle, Add, Remove, Deduplicate, RandomItem, Length' },
			{ name: 'input_var', type: 'string', required: true, description: 'Variable containing the list' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'RESULT' },
			{ name: 'param1', type: 'string', required: false, description: 'Parameter (e.g. delimiter for Join, item for Add/Remove)' },
		],
		codeExample: `Function: Join
Input: PARSED_LIST
Param1: , (comma delimiter)
→ Joins list items into a comma-separated string`,
		tips: [
			'Lists are created by recursive ParseLR or Split string function',
			'RandomItem picks a random element from the list',
			'Deduplicate removes duplicate entries',
		],
		relatedBlocks: ['StringFunction', 'ParseLR', 'Loop'],
		rustCode: `let input = self.variables.get(&settings.input_var).unwrap_or_default();
let items: Vec<String> = serde_json::from_str(&input)
    .unwrap_or_else(|_| vec![input.clone()]);

let result = match settings.function_type {
    Join => items.join(&param1),
    Sort => { let mut s = items; s.sort(); serde_json::to_string(&s)? }
    Shuffle => {
        let mut s = items;
        s.shuffle(&mut rand::thread_rng());
        serde_json::to_string(&s)?
    }
    Add => { let mut l = items; l.push(param1); serde_json::to_string(&l)? }
    Remove => serde_json::to_string(
        &items.into_iter().filter(|i| *i != param1).collect::<Vec<_>>()
    )?,
    Deduplicate => {
        let mut seen = HashSet::new();
        serde_json::to_string(
            &items.into_iter().filter(|i| seen.insert(i.clone())).collect::<Vec<_>>()
        )?
    }
    RandomItem => items.choose(&mut rand::thread_rng()).cloned().unwrap_or_default(),
    Length => items.len().to_string(),
};
self.variables.set_user(&settings.output_var, result, settings.capture);`,
	},
	{
		type: 'CryptoFunction',
		name: 'Crypto Function',
		category: 'Functions',
		description: 'Performs cryptographic operations: hashing (MD5, SHA-family), HMAC, BCrypt, Base64, AES encryption/decryption.',
		parameters: [
			{ name: 'function_type', type: 'enum', required: true, description: 'Crypto operation: MD5, SHA1, SHA256, SHA512, HMACSHA256, BCryptHash, AESEncrypt, AESDecrypt, etc.' },
			{ name: 'input_var', type: 'string', required: true, description: 'Variable containing the input data' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'HASH' },
			{ name: 'key', type: 'string', required: false, description: 'Key for HMAC/AES operations' },
		],
		codeExample: `Function: SHA256
Input: PASS
→ Stores the SHA-256 hash of the password`,
		tips: [
			'HMAC functions require a key parameter',
			'AES uses the key parameter for encryption/decryption key',
			'BCryptHash generates a bcrypt hash, BCryptVerify checks against one',
		],
		relatedBlocks: ['StringFunction', 'ConversionFunction'],
		rustCode: `let input = self.variables.get(&settings.input_var).unwrap_or_default();
let key = self.variables.interpolate(&settings.key);

let result = match settings.function_type {
    MD5 => format!("{:x}", md5::Md5::digest(input.as_bytes())),
    SHA256 => format!("{:x}", sha2::Sha256::digest(input.as_bytes())),
    SHA512 => format!("{:x}", sha2::Sha512::digest(input.as_bytes())),
    HMACSHA256 => {
        let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())?;
        mac.update(input.as_bytes());
        format!("{:x}", mac.finalize().into_bytes())
    }
    // ...other hash variants follow same pattern
};
self.variables.set_user(&settings.output_var, result, settings.capture);`,
	},
	{
		type: 'ConversionFunction',
		name: 'Conversion',
		category: 'Functions',
		description: 'Converts data between types: string, int, float, hex, base64, JSON.',
		parameters: [
			{ name: 'input_var', type: 'string', required: true, description: 'Variable containing the input value' },
			{ name: 'from_type', type: 'string', required: true, description: 'Source type', default: 'string' },
			{ name: 'to_type', type: 'string', required: true, description: 'Target type', default: 'int' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'CONVERTED' },
		],
		codeExample: `From: string  To: int
Input: RESPONSECODE
→ Converts "200" to numeric 200 for comparison`,
		tips: [
			'Useful before numeric comparisons in KeyCheck',
			'hex ↔ string conversions for binary data',
		],
		relatedBlocks: ['StringFunction', 'CryptoFunction'],
		rustCode: `let input = self.variables.get(&settings.input_var).unwrap_or_default();
let result = match (settings.from_type.as_str(), settings.to_type.as_str()) {
    ("string", "int") => input.parse::<i64>().map(|v| v.to_string()).unwrap_or_default(),
    ("string", "float") => input.parse::<f64>().map(|v| v.to_string()).unwrap_or_default(),
    ("string", "bool") => match input.to_lowercase().as_str() {
        "true" | "1" | "yes" => "true".into(),
        _ => "false".into(),
    },
    ("string", "hex") => input.as_bytes().iter().map(|b| format!("{:02x}", b)).collect(),
    ("hex", "string") => {
        let bytes: Vec<u8> = (0..input.len()).step_by(2)
            .filter_map(|i| u8::from_str_radix(&input[i..i+2], 16).ok())
            .collect();
        String::from_utf8_lossy(&bytes).to_string()
    }
    _ => input,
};
self.variables.set_user(&settings.output_var, result, settings.capture);`,
	},
	{
		type: 'DateFunction',
		name: 'Date Function',
		category: 'Functions',
		description: 'Performs date/time operations: get current time, format dates, parse date strings, add/subtract time, Unix timestamps.',
		parameters: [
			{ name: 'function_type', type: 'enum', required: true, description: 'Operation: Now, FormatDate, ParseDate, AddTime, SubtractTime, UnixTimestamp, UnixToDate' },
			{ name: 'input_var', type: 'string', required: false, description: 'Variable containing a date string to process' },
			{ name: 'format', type: 'string', required: false, description: 'Date format string (strftime syntax)', default: '%Y-%m-%d %H:%M:%S' },
			{ name: 'amount', type: 'number', required: false, description: 'Amount of time to add/subtract', default: '0' },
			{ name: 'unit', type: 'string', required: false, description: 'Time unit: seconds, minutes, hours, days', default: 'seconds' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'DATE' },
		],
		codeExample: `Function: Now
Format: %Y-%m-%d %H:%M:%S
→ Stores current date/time like "2024-01-15 14:30:00"`,
		tips: [
			'Now returns the current UTC time in the specified format',
			'UnixTimestamp gives seconds since epoch',
			'Common formats: %Y-%m-%d, %H:%M:%S, %s (unix)',
		],
		relatedBlocks: ['SetVariable', 'StringFunction'],
		rustCode: `let result = match settings.function_type {
    Now => chrono::Local::now().format(&settings.format).to_string(),
    UnixTimestamp => chrono::Utc::now().timestamp().to_string(),
    UnixToDate => {
        let ts: i64 = input.parse().unwrap_or(0);
        DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.format(&settings.format).to_string())
            .unwrap_or_default()
    }
    AddTime | SubtractTime => {
        let delta_secs = match settings.unit.as_str() {
            "minutes" => amount * 60,
            "hours" => amount * 3600,
            "days" => amount * 86400,
            _ => amount, // seconds
        };
        let new_ts = if add { ts + delta_secs } else { ts - delta_secs };
        DateTime::from_timestamp(new_ts, 0)
            .map(|dt| dt.format(&settings.format).to_string())
            .unwrap_or_else(|| new_ts.to_string())
    }
};
self.variables.set_user(&settings.output_var, result, settings.capture);`,
	},
	{
		type: 'CaseSwitch',
		name: 'Case / Switch',
		category: 'Control',
		description: 'Maps an input value to a result using case matching, like a switch statement. Checks cases in order and uses default when none match.',
		parameters: [
			{ name: 'input_var', type: 'string', required: true, description: 'Variable to match against each case', default: 'data.RESPONSECODE' },
			{ name: 'cases', type: 'array', required: true, description: 'List of match_value → result_value pairs' },
			{ name: 'default_value', type: 'string', required: false, description: 'Result when no case matches', default: 'FAIL' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'RESULT' },
		],
		codeExample: `Input: data.RESPONSECODE
200 → "SUCCESS"
403 → "BAN"
Default: "FAIL"`,
		tips: [
			'Cases are checked in order — first match wins',
			'Simpler than multiple IfElse blocks for value mapping',
			'Use with KeyCheck for status assignment based on mapped values',
		],
		relatedBlocks: ['IfElse', 'KeyCheck'],
		rustCode: `let input = self.variables.get(&settings.input_var).unwrap_or_default();
let result = settings.cases.iter()
    .find(|c| c.match_value == input)
    .map(|c| self.variables.interpolate(&c.result_value))
    .unwrap_or_else(|| self.variables.interpolate(&settings.default_value));
self.variables.set_user(&settings.output_var, result, settings.capture);`,
	},
	{
		type: 'CookieContainer',
		name: 'Cookie Container',
		category: 'Functions',
		description: 'Loads cookies from a file (Netscape format) or raw text into the session. Supports domain filtering and Netscape format export.',
		parameters: [
			{ name: 'source', type: 'string', required: true, description: 'File path or raw cookie text depending on source_type' },
			{ name: 'source_type', type: 'enum', required: false, description: '"file" to read from path, "text" to use raw input', default: 'text' },
			{ name: 'domain', type: 'string', required: false, description: 'Filter cookies to this domain only' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store cookies', default: 'COOKIES' },
			{ name: 'save_netscape', type: 'boolean', required: false, description: 'Also store in Netscape format', default: 'false' },
		],
		codeExample: `Source type: text
Source: session=abc123; domain=.example.com
Domain: example.com
→ Loads session cookie for example.com`,
		tips: [
			'Netscape format: domain, flag, path, secure, expiry, name, value (tab-separated)',
			'Use with HttpRequest to send pre-loaded cookies',
		],
		relatedBlocks: ['ParseCookie', 'ClearCookies', 'HttpRequest'],
		rustCode: `let raw_text = match settings.source_type.as_str() {
    "file" => std::fs::read_to_string(&settings.source)?,
    _ => self.variables.interpolate(&settings.source),
};
let domain = self.variables.interpolate(&settings.domain);
let mut cookies = Vec::new();
for line in raw_text.lines() {
    let parts: Vec<&str> = line.split('\\t').collect();
    if parts.len() >= 7 {
        if domain.is_empty() || parts[0].contains(&domain) {
            cookies.push((parts[5].to_string(), parts[6].to_string()));
        }
    }
}
let value = cookies.iter().map(|(n,v)| format!("{}={}", n, v)).collect::<Vec<_>>().join("; ");
self.variables.set_user(&settings.output_var, value, settings.capture);`,
	},
	{
		type: 'IfElse',
		name: 'If / Else',
		category: 'Control',
		description: 'Conditional branching: executes true_blocks if the condition is met, otherwise executes false_blocks. Supports nested blocks.',
		parameters: [
			{ name: 'condition', type: 'object', required: true, description: 'Condition with source variable, comparison operator, and value' },
			{ name: 'true_blocks', type: 'array', required: false, description: 'Blocks to execute when condition is true' },
			{ name: 'false_blocks', type: 'array', required: false, description: 'Blocks to execute when condition is false' },
		],
		codeExample: `If data.RESPONSECODE EqualTo "200"
  → Parse response, extract token
Else
  → Log error message`,
		tips: [
			'Drag blocks into the true/false branches in the visual editor',
			'Supports the same comparison operators as KeyCheck',
			'Can be nested for complex logic flows',
		],
		relatedBlocks: ['KeyCheck', 'CaseSwitch', 'Loop'],
		rustCode: `// Evaluate the condition using variable values
let source_val = self.variables.get(&settings.condition.source).unwrap_or_default();
let target = self.variables.interpolate(&settings.condition.value);
let result = match settings.condition.comparison {
    Comparison::Contains => source_val.contains(&target),
    Comparison::EqualTo => source_val == target,
    Comparison::MatchesRegex => Regex::new(&target)?.is_match(&source_val),
    Comparison::GreaterThan => source_val.parse::<f64>()? > target.parse::<f64>()?,
    Comparison::Exists => !source_val.is_empty(),
    // ... other comparisons
};
let branch = if result { &settings.true_blocks } else { &settings.false_blocks };
self.execute_blocks(branch, sidecar_tx).await`,
	},
	{
		type: 'Loop',
		name: 'Loop',
		category: 'Control',
		description: 'Repeats a set of blocks either a fixed number of times (Repeat) or once for each item in a list (ForEach).',
		parameters: [
			{ name: 'loop_type', type: 'enum', required: true, description: 'ForEach iterates over a list, Repeat runs N times', default: 'ForEach' },
			{ name: 'list_var', type: 'string', required: false, description: 'Variable containing the list to iterate (ForEach mode)' },
			{ name: 'item_var', type: 'string', required: false, description: 'Variable name for the current item', default: 'ITEM' },
			{ name: 'count', type: 'number', required: false, description: 'Number of iterations (Repeat mode)', default: '1' },
		],
		codeExample: `Type: ForEach
List: PARSED_EMAILS
Item var: EMAIL
→ Iterates over each email, accessible as <EMAIL>`,
		tips: [
			'ForEach mode needs a list variable (from recursive ParseLR or Split)',
			'Current item is available as <item_var> inside the loop',
			'Avoid infinite loops — always have a clear exit condition',
		],
		relatedBlocks: ['IfElse', 'ListFunction', 'ParseLR'],
		rustCode: `match settings.loop_type {
    LoopType::ForEach => {
        let list_str = self.variables.get(&settings.list_var).unwrap_or_default();
        // Try JSON array first, fallback to single item
        let items: Vec<String> = serde_json::from_str(&list_str)
            .unwrap_or_else(|_| vec![list_str]);
        for item in items {
            self.variables.set_user(&settings.item_var, item, false);
            self.execute_blocks(&settings.blocks, sidecar_tx).await?;
            if self.status != BotStatus::None { break; }
        }
    }
    LoopType::Repeat => {
        for _ in 0..settings.count {
            self.execute_blocks(&settings.blocks, sidecar_tx).await?;
        }
    }
}`,
	},
	{
		type: 'Delay',
		name: 'Delay',
		category: 'Control',
		description: 'Pauses execution for a random duration between min and max milliseconds. Useful for rate limiting and avoiding detection.',
		parameters: [
			{ name: 'min_ms', type: 'number', required: true, description: 'Minimum delay in milliseconds', default: '1000' },
			{ name: 'max_ms', type: 'number', required: true, description: 'Maximum delay in milliseconds', default: '1000' },
		],
		codeExample: `Min: 500ms, Max: 2000ms
→ Waits a random 0.5-2 seconds`,
		tips: [
			'Set min = max for a fixed delay',
			'Add delays between requests to avoid rate limiting',
			'Randomized delays are harder for anti-bot systems to detect',
		],
		relatedBlocks: ['HttpRequest', 'Loop'],
		rustCode: `let ms = if settings.min_ms == settings.max_ms {
    settings.min_ms
} else {
    rand::thread_rng().gen_range(settings.min_ms..=settings.max_ms)
};
tokio::time::sleep(Duration::from_millis(ms)).await;`,
	},
	{
		type: 'Script',
		name: 'Script',
		category: 'Control',
		description: 'Executes custom JavaScript code with access to pipeline variables. Use "return" to output a value to the output variable.',
		parameters: [
			{ name: 'code', type: 'string', required: true, description: 'JavaScript code to execute' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the return value', default: 'RESULT' },
		],
		codeExample: `// Generate a timestamp
return Date.now().toString();

// Access variables with vars object
let user = vars["USER"];
return user.toLowerCase();`,
		tips: [
			'Use "return" to pass a value back to the pipeline',
			'Variables are accessible via the vars object',
			'Useful for complex logic that blocks alone cannot express',
		],
		relatedBlocks: ['SetVariable', 'IfElse', 'StringFunction'],
		rustCode: `// Script execution is planned but not yet implemented.
// Will support a lightweight expression language for
// variable manipulation and control flow.
// Current workaround: use StringFunction + SetVariable blocks.
Ok(())`,
	},
	{
		type: 'Log',
		name: 'Log',
		category: 'Utilities',
		description: 'Writes a message to the debug log. Supports variable interpolation with <VAR> syntax. Essential for debugging.',
		parameters: [
			{ name: 'message', type: 'string', required: true, description: 'Log message. Use <VAR> for variable values.' },
		],
		codeExample: `Message: Response code: <data.RESPONSECODE>, Token: <TOKEN>
→ Logs "Response code: 200, Token: abc123"`,
		tips: [
			'Invaluable for debugging — add Log blocks to inspect variable values',
			'Use <VAR> syntax to interpolate any variable',
			'Check the Debug tab at the bottom to see log output',
		],
		relatedBlocks: ['SetVariable', 'Webhook'],
		rustCode: `let message = self.variables.interpolate(&settings.message);
self.log.push(LogEntry {
    timestamp_ms: elapsed_ms(),
    block_id: block.id,
    block_label: block.label.clone(),
    message,
});`,
	},
	{
		type: 'SetVariable',
		name: 'Set Variable',
		category: 'Utilities',
		description: 'Creates or updates a variable with a specific value. The value supports variable interpolation with <VAR> syntax.',
		parameters: [
			{ name: 'name', type: 'string', required: true, description: 'Variable name to set' },
			{ name: 'value', type: 'string', required: true, description: 'Value to assign. Use <VAR> for interpolation.' },
			{ name: 'capture', type: 'boolean', required: false, description: 'Mark as captured output', default: 'false' },
		],
		codeExample: `Name: FULL_URL
Value: https://example.com/api/<TOKEN>
→ Creates FULL_URL with the token interpolated`,
		tips: [
			'Variables set here are available to all subsequent blocks',
			'Enable "capture" to include in hit output',
			'Use for building dynamic URLs, combining extracted data',
		],
		relatedBlocks: ['Log', 'Script', 'StringFunction'],
		rustCode: `let value = self.variables.interpolate(&settings.value);
self.variables.set_user(&settings.name, value, settings.capture);`,
	},
	{
		type: 'ClearCookies',
		name: 'Clear Cookies',
		category: 'Utilities',
		description: 'Clears all cookies stored in the current session. No parameters needed. Useful between login attempts or when testing different accounts.',
		parameters: [],
		codeExample: `[No parameters]
→ Clears all session cookies`,
		tips: [
			'Use before a new login attempt to start with a clean cookie state',
			'Cookies accumulate across HttpRequest blocks within a session',
		],
		relatedBlocks: ['HttpRequest', 'ParseCookie', 'CookieContainer'],
		rustCode: `// Clears the session cookie jar maintained by the sidecar process.
// After this block, subsequent HTTP requests start with no cookies.
sidecar_tx.send(SidecarRequest {
    action: "clear_cookies".into(),
    ..Default::default()
}).await?;`,
	},
	{
		type: 'Webhook',
		name: 'Webhook',
		category: 'Utilities',
		description: 'Sends data to an external webhook URL. Commonly used with Discord, Slack, or custom notification endpoints.',
		parameters: [
			{ name: 'url', type: 'string', required: true, description: 'Webhook endpoint URL' },
			{ name: 'method', type: 'string', required: false, description: 'HTTP method', default: 'POST' },
			{ name: 'headers', type: 'array', required: false, description: 'Custom headers' },
			{ name: 'body_template', type: 'string', required: false, description: 'JSON body template with <VAR> interpolation' },
			{ name: 'content_type', type: 'string', required: false, description: 'Content-Type header', default: 'application/json' },
		],
		codeExample: `URL: https://discord.com/api/webhooks/...
Body: {"content": "Hit: <USER>:<PASS>"}
→ Sends a Discord notification on hit`,
		tips: [
			'Use inside a KeyCheck Success branch to notify on hits',
			'Body supports <VAR> interpolation for dynamic content',
			'Discord webhooks expect JSON with a "content" field',
		],
		relatedBlocks: ['HttpRequest', 'KeyCheck', 'Log'],
		rustCode: `let url = self.variables.interpolate(&settings.url);
let body = self.variables.interpolate(&settings.body_template);
let headers = settings.headers.iter()
    .map(|(k, v)| vec![self.variables.interpolate(k), self.variables.interpolate(v)])
    .collect();
// Fire-and-forget HTTP request via sidecar
let req = SidecarRequest {
    action: "request".into(), method: Some(settings.method.clone()),
    url: Some(url), headers: Some(headers), body: Some(body), ..
};
sidecar_tx.send(req).await?;`,
	},
	{
		type: 'WebSocket',
		name: 'WebSocket',
		category: 'Utilities',
		description: 'Manages WebSocket connections: connect, send messages, receive responses, or close the connection.',
		parameters: [
			{ name: 'url', type: 'string', required: true, description: 'WebSocket URL (ws:// or wss://)' },
			{ name: 'action', type: 'enum', required: true, description: 'Operation: connect, send, receive, close', default: 'connect' },
			{ name: 'message', type: 'string', required: false, description: 'Message to send (for send action)' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store received data', default: 'WS_RESPONSE' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Timeout for receive operation', default: '10000' },
		],
		codeExample: `Action: connect → ws://example.com/socket
Action: send → {"type":"auth","token":"<TOKEN>"}
Action: receive → stores response in WS_RESPONSE`,
		tips: [
			'Always connect before sending or receiving',
			'Close the connection when done to free resources',
			'Use multiple WebSocket blocks in sequence for full conversations',
		],
		relatedBlocks: ['HttpRequest', 'TcpRequest'],
		rustCode: `let url = self.variables.interpolate(&settings.url);
match settings.action.as_str() {
    "connect" => {
        let (ws, _) = tokio_tungstenite::connect_async(&url).await?;
        self.ws_connection = Some(ws);
    }
    "send" => {
        let msg = self.variables.interpolate(&settings.message);
        self.ws_connection.as_mut()?.send(Message::Text(msg)).await?;
    }
    "receive" => {
        let msg = self.ws_connection.as_mut()?.next().await??;
        self.variables.set_user(&settings.output_var, msg.to_string(), false);
    }
    "close" => { self.ws_connection.take(); }
}`,
	},
	{
		type: 'TcpRequest',
		name: 'TCP Request',
		category: 'Requests',
		description: 'Sends raw data over a TCP connection and reads the response. Supports TLS for encrypted connections.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'Target hostname or IP' },
			{ name: 'port', type: 'number', required: true, description: 'Target port number', default: '80' },
			{ name: 'data', type: 'string', required: false, description: 'Raw data to send' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store response', default: 'TCP_RESPONSE' },
			{ name: 'use_tls', type: 'boolean', required: false, description: 'Use TLS encryption', default: 'false' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Connection timeout', default: '5000' },
		],
		codeExample: `Host: smtp.example.com
Port: 25
Data: EHLO example.com\\r\\n`,
		tips: [
			'Use \\r\\n for line endings in text protocols',
			'Enable TLS for secure connections (port 443, 993, etc.)',
		],
		relatedBlocks: ['UdpRequest', 'HttpRequest', 'SshRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let mut stream = TcpStream::connect(&addr).await?;
let data = self.variables.interpolate(&settings.data);
stream.write_all(data.as_bytes()).await?;
let mut buf = vec![0u8; 8192];
let n = tokio::time::timeout(
    Duration::from_millis(settings.timeout_ms),
    stream.read(&mut buf),
).await??;
let response = String::from_utf8_lossy(&buf[..n]).to_string();
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'UdpRequest',
		name: 'UDP Request',
		category: 'Requests',
		description: 'Sends a UDP datagram and optionally reads a response. Useful for DNS queries and other UDP protocols.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'Target hostname or IP' },
			{ name: 'port', type: 'number', required: true, description: 'Target port', default: '53' },
			{ name: 'data', type: 'string', required: false, description: 'Raw data to send' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store response', default: 'UDP_RESPONSE' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Timeout in ms', default: '5000' },
		],
		codeExample: `Host: 8.8.8.8, Port: 53
→ Send DNS query and read response`,
		tips: ['UDP is connectionless — no guarantee of delivery', 'Useful for DNS, SNMP, and other UDP-based protocols'],
		relatedBlocks: ['TcpRequest', 'HttpRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let socket = UdpSocket::bind("0.0.0.0:0").await?;
let data = self.variables.interpolate(&settings.data);
socket.send_to(data.as_bytes(), &addr).await?;
let mut buf = vec![0u8; 8192];
let n = tokio::time::timeout(
    Duration::from_millis(settings.timeout_ms),
    socket.recv(&mut buf),
).await??;
let response = String::from_utf8_lossy(&buf[..n]).to_string();
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'FtpRequest',
		name: 'FTP Request',
		category: 'Requests',
		description: 'Connects to an FTP server and executes a command. Supports login with username/password.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'FTP server hostname' },
			{ name: 'port', type: 'number', required: true, description: 'FTP port', default: '21' },
			{ name: 'username', type: 'string', required: true, description: 'FTP username' },
			{ name: 'password', type: 'string', required: true, description: 'FTP password' },
			{ name: 'command', type: 'string', required: false, description: 'FTP command to execute', default: 'LIST' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store response', default: 'FTP_RESPONSE' },
		],
		codeExample: `Host: ftp.example.com
Username: <USER>, Password: <PASS>
Command: LIST
→ Lists directory contents after login`,
		tips: ['Common commands: LIST, STAT, PWD, SIZE filename'],
		relatedBlocks: ['SshRequest', 'TcpRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let mut stream = TcpStream::connect(&addr).await?;
let _ = read_response(&mut stream).await?; // 220 welcome
send_cmd(&mut stream, &format!("USER {}\\r\\n", settings.username)).await?;
send_cmd(&mut stream, &format!("PASS {}\\r\\n", settings.password)).await?;
let command = self.variables.interpolate(&settings.command);
let response = send_cmd(&mut stream, &format!("{}\\r\\n", command)).await?;
send_cmd(&mut stream, "QUIT\\r\\n").await?;
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'SshRequest',
		name: 'SSH Request',
		category: 'Requests',
		description: 'Connects to an SSH server and executes a remote command.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'SSH server hostname' },
			{ name: 'port', type: 'number', required: true, description: 'SSH port', default: '22' },
			{ name: 'username', type: 'string', required: true, description: 'SSH username' },
			{ name: 'password', type: 'string', required: true, description: 'SSH password' },
			{ name: 'command', type: 'string', required: true, description: 'Remote command to execute' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store output', default: 'SSH_RESPONSE' },
		],
		codeExample: `Host: server.example.com
Username: <USER>, Password: <PASS>
Command: whoami
→ Executes "whoami" and stores output`,
		tips: ['Connection will timeout if credentials are wrong', 'Output includes both stdout and stderr'],
		relatedBlocks: ['TcpRequest', 'FtpRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let mut stream = TcpStream::connect(&addr).await?;
// SSH transport negotiation + password auth
let command = self.variables.interpolate(&settings.command);
// Execute command and capture stdout
let response = ssh_exec(&mut stream, &settings.username, &settings.password, &command).await?;
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'ImapRequest',
		name: 'IMAP Request',
		category: 'Requests',
		description: 'Connects to an IMAP mail server to check or fetch email.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'IMAP server hostname' },
			{ name: 'port', type: 'number', required: true, description: 'IMAP port', default: '993' },
			{ name: 'username', type: 'string', required: true, description: 'Email username' },
			{ name: 'password', type: 'string', required: true, description: 'Email password' },
			{ name: 'use_tls', type: 'boolean', required: false, description: 'Use TLS encryption', default: 'true' },
			{ name: 'command', type: 'string', required: false, description: 'IMAP command', default: 'LOGIN' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store response', default: 'IMAP_RESPONSE' },
		],
		codeExample: `Host: imap.gmail.com:993
Username: <USER>, Password: <PASS>
Command: LOGIN
→ Attempts IMAP login to verify credentials`,
		tips: ['Port 993 = IMAPS (TLS), Port 143 = IMAP (plain)'],
		relatedBlocks: ['SmtpRequest', 'PopRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let mut stream = TcpStream::connect(&addr).await?;
let _ = read_line(&mut stream).await?; // * OK banner
send_cmd(&mut stream, &format!("A1 LOGIN {} {}\\r\\n", settings.username, settings.password)).await?;
let command = self.variables.interpolate(&settings.command);
let response = send_cmd(&mut stream, &format!("A2 {}\\r\\n", command)).await?;
send_cmd(&mut stream, "A3 LOGOUT\\r\\n").await?;
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'SmtpRequest',
		name: 'SMTP Request',
		category: 'Requests',
		description: 'Connects to an SMTP server to send email or verify credentials.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'SMTP server hostname' },
			{ name: 'port', type: 'number', required: true, description: 'SMTP port', default: '587' },
			{ name: 'username', type: 'string', required: true, description: 'SMTP username' },
			{ name: 'password', type: 'string', required: true, description: 'SMTP password' },
			{ name: 'use_tls', type: 'boolean', required: false, description: 'Use TLS', default: 'true' },
			{ name: 'command', type: 'string', required: false, description: 'SMTP command', default: 'EHLO' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store response', default: 'SMTP_RESPONSE' },
		],
		codeExample: `Host: smtp.gmail.com:587
Command: EHLO
→ Initiates SMTP handshake`,
		tips: ['Port 587 = submission (STARTTLS), Port 465 = SMTPS, Port 25 = plain'],
		relatedBlocks: ['ImapRequest', 'PopRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let mut stream = TcpStream::connect(&addr).await?;
let _ = read_line(&mut stream).await?; // 220 banner
send_cmd(&mut stream, "EHLO reqflow\\r\\n").await?;
// AUTH LOGIN if credentials provided
if !settings.username.is_empty() {
    send_cmd(&mut stream, "AUTH LOGIN\\r\\n").await?;
    send_cmd(&mut stream, &format!("{}\\r\\n", base64_encode(&settings.username))).await?;
    send_cmd(&mut stream, &format!("{}\\r\\n", base64_encode(&settings.password))).await?;
}
let command = self.variables.interpolate(&settings.command);
let response = send_cmd(&mut stream, &format!("{}\\r\\n", command)).await?;
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'PopRequest',
		name: 'POP Request',
		category: 'Requests',
		description: 'Connects to a POP3 mail server to retrieve email or verify credentials.',
		parameters: [
			{ name: 'host', type: 'string', required: true, description: 'POP3 server hostname' },
			{ name: 'port', type: 'number', required: true, description: 'POP3 port', default: '995' },
			{ name: 'username', type: 'string', required: true, description: 'Email username' },
			{ name: 'password', type: 'string', required: true, description: 'Email password' },
			{ name: 'use_tls', type: 'boolean', required: false, description: 'Use TLS', default: 'true' },
			{ name: 'command', type: 'string', required: false, description: 'POP3 command', default: 'STAT' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store response', default: 'POP_RESPONSE' },
		],
		codeExample: `Host: pop.gmail.com:995
Command: STAT
→ Returns mailbox statistics (message count, size)`,
		tips: ['STAT returns message count, LIST shows individual messages'],
		relatedBlocks: ['ImapRequest', 'SmtpRequest'],
		rustCode: `let host = self.variables.interpolate(&settings.host);
let addr = format!("{}:{}", host, settings.port);
let mut stream = TcpStream::connect(&addr).await?;
let _ = read_line(&mut stream).await?; // +OK banner
send_cmd(&mut stream, &format!("USER {}\\r\\n", settings.username)).await?;
send_cmd(&mut stream, &format!("PASS {}\\r\\n", settings.password)).await?;
let command = self.variables.interpolate(&settings.command);
let response = send_cmd(&mut stream, &format!("{}\\r\\n", command)).await?;
send_cmd(&mut stream, "QUIT\\r\\n").await?;
self.variables.set_user(&settings.output_var, response, settings.capture);`,
	},
	{
		type: 'CaptchaSolver',
		name: 'Captcha Solver',
		category: 'Bypass',
		description: 'Solves captchas using a third-party solver service API. Supports reCAPTCHA v2, hCaptcha, FunCaptcha, Image Captcha, and Cloudflare Turnstile.',
		parameters: [
			{ name: 'solver_service', type: 'enum', required: true, description: 'Service: capsolver, 2captcha, anticaptcha, capmonster' },
			{ name: 'captcha_type', type: 'enum', required: true, description: 'Type: RecaptchaV2, HCaptcha, FunCaptcha, ImageCaptcha, Turnstile' },
			{ name: 'api_key', type: 'string', required: true, description: 'Your solver service API key' },
			{ name: 'site_key', type: 'string', required: true, description: 'Captcha site key from the page HTML' },
			{ name: 'page_url', type: 'string', required: true, description: 'URL of the page with the captcha' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the solved token', default: 'CAPTCHA_TOKEN' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Max wait time for solution', default: '120000' },
		],
		codeExample: `Service: capsolver
Type: RecaptchaV2
Site key: 6Le... (from page source)
Page URL: https://example.com/login
→ Returns captcha token in CAPTCHA_TOKEN`,
		tips: [
			'Find the site key in the page source (data-sitekey attribute)',
			'Token is typically submitted as g-recaptcha-response form field',
			'Tokens expire quickly — use immediately after solving',
		],
		relatedBlocks: ['HttpRequest', 'CloudflareBypass', 'OcrCaptcha'],
		rustCode: `let api_key = self.variables.interpolate(&settings.api_key);
let site_key = self.variables.interpolate(&settings.site_key);
let page_url = self.variables.interpolate(&settings.page_url);
// Submit task to solver service (2captcha, capsolver, etc.)
let task_id = submit_captcha_task(&settings.solver_service, &api_key, &site_key, &page_url, &settings.captcha_type).await?;
// Poll for result with timeout
let result = poll_captcha_result(&settings.solver_service, &api_key, &task_id, settings.timeout_ms).await?;
self.variables.set_user(&settings.output_var, result, settings.capture);`,
	},
	{
		type: 'CloudflareBypass',
		name: 'Cloudflare Bypass',
		category: 'Bypass',
		description: 'Bypasses Cloudflare protection using a FlareSolverr instance. Returns cookies that can be used in subsequent requests.',
		parameters: [
			{ name: 'url', type: 'string', required: true, description: 'URL of the Cloudflare-protected page' },
			{ name: 'flaresolverr_url', type: 'string', required: false, description: 'FlareSolverr API endpoint', default: 'http://localhost:8191/v1' },
			{ name: 'max_timeout_ms', type: 'number', required: false, description: 'Maximum wait time', default: '60000' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store bypass cookies', default: 'CF_COOKIES' },
		],
		codeExample: `URL: https://protected-site.com
FlareSolverr: http://localhost:8191/v1
→ Returns CF clearance cookies`,
		tips: [
			'Requires a running FlareSolverr instance (Docker recommended)',
			'Cookies from bypass should be sent with subsequent HttpRequest blocks',
		],
		relatedBlocks: ['CaptchaSolver', 'HttpRequest'],
		rustCode: `let url = self.variables.interpolate(&settings.url);
let flaresolverr = self.variables.interpolate(&settings.flaresolverr_url);
// POST to FlareSolverr API
let body = serde_json::json!({
    "cmd": "request.get", "url": url,
    "maxTimeout": settings.max_timeout_ms,
});
let resp = reqwest::Client::new()
    .post(&format!("{}/v1", flaresolverr))
    .json(&body).send().await?.json::<Value>().await?;
let solution = &resp["solution"];
let cookies = solution["cookies"].as_str().unwrap_or("");
self.variables.set_user(&settings.output_var, cookies.to_string(), settings.capture);`,
	},
	{
		type: 'LaravelCsrf',
		name: 'Laravel CSRF',
		category: 'Bypass',
		description: 'Fetches a Laravel CSRF token from a page. Extracts both the hidden input token and the XSRF cookie.',
		parameters: [
			{ name: 'url', type: 'string', required: true, description: 'URL of the page with the CSRF token' },
			{ name: 'csrf_selector', type: 'string', required: false, description: 'CSS selector for the hidden input', default: 'input[name="_token"]' },
			{ name: 'cookie_name', type: 'string', required: false, description: 'Name of the CSRF cookie', default: 'XSRF-TOKEN' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the token', default: 'CSRF_TOKEN' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Request timeout', default: '10000' },
		],
		codeExample: `URL: https://laravel-app.com/login
→ Extracts _token and XSRF-TOKEN cookie
→ Use <CSRF_TOKEN> in subsequent POST body`,
		tips: [
			'The CSRF token must be included in form submissions',
			'Token changes on each page load — always fetch fresh',
		],
		relatedBlocks: ['HttpRequest', 'ParseCSS'],
		rustCode: `let url = self.variables.interpolate(&settings.url);
// GET the page via sidecar
let req = SidecarRequest { action: "request".into(), method: Some("GET".into()), url: Some(url), .. };
let resp = sidecar_tx.send(req).await?;
// Extract CSRF token using CSS selector (scraper crate)
let document = scraper::Html::parse_document(&resp.body);
let selector = scraper::Selector::parse(&settings.csrf_selector)?;
let token = document.select(&selector).next()
    .and_then(|el| el.value().attr("value").or(el.value().attr("content")))
    .unwrap_or("").to_string();
self.variables.set_user(&settings.output_var, token, settings.capture);`,
	},
	{
		type: 'BrowserOpen',
		name: 'Browser Open',
		category: 'Browser',
		description: 'Opens a headless or visible browser instance for automation. Must be called before any other browser blocks.',
		parameters: [
			{ name: 'headless', type: 'boolean', required: false, description: 'Run without visible window', default: 'true' },
			{ name: 'browser_type', type: 'string', required: false, description: 'Browser engine: chromium, firefox, webkit', default: 'chromium' },
			{ name: 'proxy', type: 'string', required: false, description: 'Proxy URL for browser traffic' },
			{ name: 'extra_args', type: 'string', required: false, description: 'Extra command-line arguments' },
		],
		codeExample: `Browser: chromium
Headless: true
→ Opens a headless Chrome instance`,
		tips: [
			'Always pair with browser blocks like NavigateTo, ClickElement',
			'Headless = false is useful for debugging (shows the browser)',
			'Extra args: --disable-gpu, --no-sandbox',
		],
		relatedBlocks: ['NavigateTo', 'ClickElement', 'TypeText'],
		rustCode: `let mut builder = chromiumoxide::BrowserConfig::builder();
if !settings.headless { builder = builder.with_head(); }
if !settings.proxy.is_empty() {
    builder = builder.arg(format!("--proxy-server={}", self.variables.interpolate(&settings.proxy)));
}
for arg in settings.extra_args.split_whitespace() {
    builder = builder.arg(arg);
}
let (browser, mut handler) = Browser::launch(builder.build()?).await?;
tokio::spawn(async move { while handler.next().await.is_some() {} });
self.browser = Some(browser);`,
	},
	{
		type: 'NavigateTo',
		name: 'Navigate To',
		category: 'Browser',
		description: 'Navigates the browser to a URL and waits for the page to load.',
		parameters: [
			{ name: 'url', type: 'string', required: true, description: 'URL to navigate to' },
			{ name: 'wait_until', type: 'string', required: false, description: 'Load state to wait for: load, domcontentloaded, networkidle', default: 'load' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Navigation timeout', default: '30000' },
		],
		codeExample: `URL: https://example.com/login
Wait until: networkidle
→ Navigates and waits for all network activity to stop`,
		tips: [
			'networkidle waits for no network activity for 500ms — most reliable',
			'domcontentloaded is faster but may miss async-loaded content',
		],
		relatedBlocks: ['BrowserOpen', 'ClickElement', 'WaitForElement'],
		rustCode: `let browser = self.browser.as_ref().ok_or("No browser open")?;
let url = self.variables.interpolate(&settings.url);
let page = browser.new_page(&url).await?;
let _ = page.wait_for_navigation().await;
let content = page.content().await.unwrap_or_default();
self.variables.set_data("SOURCE", content);
self.variables.set_data("URL", page.url().await?.to_string());
self.page = Some(page);`,
	},
	{
		type: 'ClickElement',
		name: 'Click Element',
		category: 'Browser',
		description: 'Clicks an element on the page matching the CSS selector.',
		parameters: [
			{ name: 'selector', type: 'string', required: true, description: 'CSS selector for the element to click' },
			{ name: 'wait_for_navigation', type: 'boolean', required: false, description: 'Wait for page navigation after click', default: 'false' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Timeout for finding the element', default: '5000' },
		],
		codeExample: `Selector: button[type="submit"]
Wait for navigation: true
→ Clicks the submit button and waits for page load`,
		tips: [
			'Enable wait_for_navigation for login/form submit buttons',
			'Use WaitForElement first if the element loads dynamically',
		],
		relatedBlocks: ['TypeText', 'WaitForElement', 'NavigateTo'],
		rustCode: `let page = self.page.as_ref().ok_or("No page open")?;
let selector = self.variables.interpolate(&settings.selector);
let element = page.find_element(&selector).await?;
element.click().await?;
if settings.wait_for_navigation {
    let _ = page.wait_for_navigation().await;
    self.variables.set_data("SOURCE", page.content().await?);
}`,
	},
	{
		type: 'TypeText',
		name: 'Type Text',
		category: 'Browser',
		description: 'Types text into an input field with optional keystroke delay for realistic typing simulation.',
		parameters: [
			{ name: 'selector', type: 'string', required: true, description: 'CSS selector for the input field' },
			{ name: 'text', type: 'string', required: true, description: 'Text to type. Supports <VAR> interpolation' },
			{ name: 'clear_first', type: 'boolean', required: false, description: 'Clear existing text before typing', default: 'true' },
			{ name: 'delay_ms', type: 'number', required: false, description: 'Delay between keystrokes', default: '50' },
		],
		codeExample: `Selector: input[name="email"]
Text: <USER>
Clear first: true
→ Types the username into the email field`,
		tips: [
			'Use a small delay (30-100ms) for realistic typing',
			'Clear first prevents appending to existing text',
		],
		relatedBlocks: ['ClickElement', 'WaitForElement'],
		rustCode: `let page = self.page.as_ref().ok_or("No page open")?;
let selector = self.variables.interpolate(&settings.selector);
let text = self.variables.interpolate(&settings.text);
let element = page.find_element(&selector).await?;
if settings.clear_first {
    element.click().await?;
    // Ctrl+A then Backspace to clear
    page.execute(DispatchKeyEventParams { key: "a", modifiers: 2 }).await?;
    page.execute(DispatchKeyEventParams { key: "Backspace" }).await?;
}
element.type_str(&text).await?;`,
	},
	{
		type: 'WaitForElement',
		name: 'Wait For Element',
		category: 'Browser',
		description: 'Waits for an element to appear, disappear, or reach a specific state on the page.',
		parameters: [
			{ name: 'selector', type: 'string', required: true, description: 'CSS selector for the element' },
			{ name: 'state', type: 'string', required: false, description: 'State to wait for: visible, hidden, attached, detached', default: 'visible' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Maximum wait time', default: '10000' },
		],
		codeExample: `Selector: .dashboard-content
State: visible
→ Waits for the dashboard to become visible after login`,
		tips: [
			'visible = element exists and is not hidden',
			'Use before interacting with dynamically loaded elements',
		],
		relatedBlocks: ['ClickElement', 'GetElementText', 'NavigateTo'],
		rustCode: `let page = self.page.as_ref().ok_or("No page open")?;
let selector = self.variables.interpolate(&settings.selector);
let timeout = Duration::from_millis(settings.timeout_ms);
let start = Instant::now();
loop {
    if page.find_element(&selector).await.is_ok() { break; }
    if start.elapsed() > timeout {
        return Err(format!("Timeout waiting for '{}'", selector));
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
}`,
	},
	{
		type: 'GetElementText',
		name: 'Get Element Text',
		category: 'Browser',
		description: 'Reads text content or a specific attribute from a page element.',
		parameters: [
			{ name: 'selector', type: 'string', required: true, description: 'CSS selector for the element' },
			{ name: 'attribute', type: 'string', required: false, description: 'HTML attribute to read. Empty = inner text' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the text', default: 'ELEMENT_TEXT' },
		],
		codeExample: `Selector: span.user-email
Attribute: (empty)
→ Reads the text content of the user email span`,
		tips: [
			'Leave attribute empty to get visible text content',
			'Use "value" attribute for input fields, "href" for links',
		],
		relatedBlocks: ['WaitForElement', 'ExecuteJs', 'Screenshot'],
		rustCode: `let page = self.page.as_ref().ok_or("No page open")?;
let selector = self.variables.interpolate(&settings.selector);
let element = page.find_element(&selector).await?;
let value = if settings.attribute.is_empty() || settings.attribute == "innerText" {
    element.inner_text().await?.unwrap_or_default()
} else {
    element.attribute(&settings.attribute).await?.unwrap_or_default()
};
self.variables.set_user(&settings.output_var, value, settings.capture);`,
	},
	{
		type: 'Screenshot',
		name: 'Screenshot',
		category: 'Browser',
		description: 'Takes a screenshot of the page or a specific element. Stores the image as a base64-encoded string.',
		parameters: [
			{ name: 'full_page', type: 'boolean', required: false, description: 'Capture the entire scrollable page', default: 'false' },
			{ name: 'selector', type: 'string', required: false, description: 'CSS selector for a specific element (empty = full viewport)' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store base64 image', default: 'SCREENSHOT_B64' },
		],
		codeExample: `Full page: false
Selector: .captcha-image
→ Captures just the captcha image element`,
		tips: [
			'Output is base64-encoded PNG — can be used with OcrCaptcha',
			'Full page captures everything including off-screen content',
		],
		relatedBlocks: ['OcrCaptcha', 'WaitForElement', 'ExecuteJs'],
		rustCode: `let page = self.page.as_ref().ok_or("No page open")?;
let bytes = if !settings.selector.is_empty() {
    let el = page.find_element(&self.variables.interpolate(&settings.selector)).await?;
    el.screenshot(CaptureScreenshotFormat::Png).await?
} else {
    page.screenshot(CaptureScreenshotParams::builder()
        .format(CaptureScreenshotFormat::Png).build()?).await?
};
let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
self.variables.set_user(&settings.output_var, b64, false);`,
	},
	{
		type: 'ExecuteJs',
		name: 'Execute JS',
		category: 'Browser',
		description: 'Executes JavaScript code in the browser page context. Can interact with the DOM and return values.',
		parameters: [
			{ name: 'code', type: 'string', required: true, description: 'JavaScript code to execute in the page' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the return value', default: 'JS_RESULT' },
		],
		codeExample: `// Get the CSRF token from a meta tag
return document.querySelector('meta[name="csrf-token"]').content;`,
		tips: [
			'Code runs in the page context — full DOM access',
			'Use "return" to pass values back to the pipeline',
			'Unlike the Script block, this runs IN the browser page',
		],
		relatedBlocks: ['Script', 'GetElementText', 'NavigateTo'],
		rustCode: `let page = self.page.as_ref().ok_or("No page open")?;
let code = self.variables.interpolate(&settings.code);
let result = page.evaluate_expression(&code).await?;
let value = match result.value() {
    Some(Value::String(s)) => s.clone(),
    Some(other) => other.to_string(),
    None => String::new(),
};
self.variables.set_user(&settings.output_var, value, settings.capture);`,
	},
	{
		type: 'RandomUserAgent',
		name: 'Random User Agent',
		category: 'Utilities',
		description: 'Generates a random, realistic User-Agent string. Can filter by browser and platform, or use a custom list.',
		parameters: [
			{ name: 'mode', type: 'enum', required: true, description: 'Random = generate realistic UA, CustomList = pick from your list', default: 'Random' },
			{ name: 'browser_filter', type: 'array', required: false, description: 'Browsers to include: Chrome, Firefox, Safari, Edge' },
			{ name: 'platform_filter', type: 'array', required: false, description: 'Platforms: Desktop, Mobile' },
			{ name: 'custom_list', type: 'string', required: false, description: 'Custom UA strings, one per line (CustomList mode)' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the User-Agent', default: 'USER_AGENT' },
			{ name: 'match_tls', type: 'boolean', required: false, description: 'Match TLS fingerprint to the generated UA', default: 'false' },
		],
		codeExample: `Mode: Random
Browsers: Chrome, Firefox
Platform: Desktop
→ Generates a realistic desktop Chrome/Firefox User-Agent`,
		tips: [
			'Use <USER_AGENT> in the User-Agent header of HttpRequest',
			'match_tls adjusts TLS fingerprint to match the browser',
			'Custom list mode picks a random line from your list',
		],
		relatedBlocks: ['HttpRequest', 'BrowserOpen'],
		rustCode: `let ua = match settings.mode {
    Mode::Random => {
        BUILTIN_USER_AGENTS.iter()
            .filter(|(_, browser, platform)| {
                (settings.browser_filter.is_empty() || settings.browser_filter.contains(browser))
                && (settings.platform_filter.is_empty() || settings.platform_filter.contains(platform))
            })
            .choose(&mut rand::thread_rng())
            .map(|(ua, _, _)| ua.to_string())
            .unwrap_or_default()
    }
    Mode::CustomList => {
        settings.custom_list.lines().choose(&mut rand::thread_rng())
            .unwrap_or("").to_string()
    }
};
self.variables.set_user(&settings.output_var, ua, settings.capture);`,
	},
	{
		type: 'OcrCaptcha',
		name: 'OCR Captcha',
		category: 'Bypass',
		description: 'Performs OCR (optical character recognition) on a base64-encoded image to read text captchas locally without a third-party service.',
		parameters: [
			{ name: 'input_var', type: 'string', required: true, description: 'Variable containing base64 image data', default: 'SCREENSHOT_B64' },
			{ name: 'language', type: 'string', required: false, description: 'OCR language', default: 'eng' },
			{ name: 'psm', type: 'number', required: false, description: 'Page segmentation mode (Tesseract)', default: '7' },
			{ name: 'whitelist', type: 'string', required: false, description: 'Allowed characters (e.g. 0123456789 for numbers only)' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store OCR text', default: 'OCR_TEXT' },
		],
		codeExample: `Input: SCREENSHOT_B64
Whitelist: 0123456789ABCDEF
→ Reads text from captcha image, only allowing hex chars`,
		tips: [
			'PSM 7 = single text line — best for simple captchas',
			'Whitelist restricts recognized characters for better accuracy',
			'Works best on clean, high-contrast captcha images',
		],
		relatedBlocks: ['Screenshot', 'CaptchaSolver'],
		rustCode: `let input_b64 = self.variables.get(&settings.input_var).unwrap_or_default();
let image_bytes = base64::decode(&input_b64)?;
let temp_path = std::env::temp_dir().join(format!("ocr_{}.png", Uuid::new_v4()));
std::fs::write(&temp_path, &image_bytes)?;
let mut args = rusty_tesseract::Args::default();
args.lang = settings.language.clone();
args.psm = Some(settings.psm as i32);
let image = rusty_tesseract::Image::from_path(&temp_path)?;
let result = rusty_tesseract::image_to_string(&image, &args)?;
std::fs::remove_file(&temp_path).ok();
self.variables.set_user(&settings.output_var, result.trim().to_string(), settings.capture);`,
	},
	{
		type: 'RecaptchaInvisible',
		name: 'reCAPTCHA Invisible',
		category: 'Bypass',
		description: 'Generates a reCAPTCHA invisible token by directly interacting with Google reCAPTCHA API endpoints.',
		parameters: [
			{ name: 'sitekey', type: 'string', required: true, description: 'reCAPTCHA site key' },
			{ name: 'anchor_url', type: 'string', required: true, description: 'Anchor URL from the reCAPTCHA iframe' },
			{ name: 'reload_url', type: 'string', required: true, description: 'Reload URL for token generation' },
			{ name: 'action', type: 'string', required: false, description: 'reCAPTCHA action parameter' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the token', default: 'RECAPTCHA_TOKEN' },
		],
		codeExample: `Sitekey: 6Le...
Anchor URL: (from network tab)
→ Generates reCAPTCHA token without a solver service`,
		tips: [
			'Capture the anchor and reload URLs from the browser network tab',
			'No solver service needed — direct API interaction',
		],
		relatedBlocks: ['CaptchaSolver', 'HttpRequest'],
		rustCode: `// Step 1: GET anchor URL to extract recaptcha-token
let anchor_resp = sidecar.request("GET", &settings.anchor_url, &settings.user_agent).await?;
let token = extract_between(&anchor_resp.body, "recaptcha-token\\" value=\\"", "\\"");
// Step 2: POST reload URL with token parameters
let post_body = format!("v={}&reason=q&c={}&k={}&co={}&hl=en&size={}&cb={}&sa={}",
    settings.v, token, settings.sitekey, settings.co, settings.size, settings.cb, settings.action);
let reload_resp = sidecar.request("POST", &settings.reload_url, &post_body).await?;
let rresp = extract_between(&reload_resp.body, "rresp\\":\\"", "\\"");
self.variables.set_user(&settings.output_var, rresp, settings.capture);`,
	},
	{
		type: 'XacfSensor',
		name: 'XACF Sensor',
		category: 'Sensors',
		description: 'Generates an XACF sensor data payload for Akamai-protected websites.',
		parameters: [
			{ name: 'bundle_id', type: 'string', required: true, description: 'Akamai bundle ID from the target site' },
			{ name: 'version', type: 'string', required: false, description: 'Sensor version', default: '2.1.2' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store sensor data', default: 'XACF_SENSOR' },
		],
		codeExample: `Bundle ID: abc123...
→ Generates sensor payload to submit as _abck cookie`,
		tips: [
			'Bundle ID is found in the Akamai script on the target page',
			'Submit the sensor data in the request to pass Akamai checks',
		],
		relatedBlocks: ['DataDomeSensor', 'HttpRequest'],
		rustCode: `// Generate Akamai Bot Manager sensor data
let bundle_id = self.variables.interpolate(&settings.bundle_id);
let version = self.variables.interpolate(&settings.version);
let sensor = generate_xacf_sensor_data(&bundle_id, &version);
// Sensor includes randomized touch events, accelerometer data,
// screen dimensions, and timing values
self.variables.set_user(&settings.output_var, sensor, settings.capture);`,
	},
	{
		type: 'DataDomeSensor',
		name: 'DataDome Sensor',
		category: 'Sensors',
		description: 'Generates a DataDome sensor payload to bypass DataDome bot protection.',
		parameters: [
			{ name: 'site_url', type: 'string', required: true, description: 'Target site URL' },
			{ name: 'cookie_datadome', type: 'string', required: true, description: 'Current DataDome cookie value' },
			{ name: 'user_agent', type: 'string', required: true, description: 'User-Agent string to match' },
			{ name: 'custom_wasm_b64', type: 'string', required: false, description: 'Custom WASM binary (base64)' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store sensor data', default: 'DD_SENSOR' },
		],
		codeExample: `Site URL: https://protected-site.com
Cookie: datadome=xxx
→ Generates sensor payload for DataDome validation`,
		tips: [
			'Requires the current datadome cookie from initial page load',
			'User-Agent must match what you use in HttpRequest headers',
		],
		relatedBlocks: ['XacfSensor', 'HttpRequest', 'RandomUserAgent'],
		rustCode: `let site_url = self.variables.interpolate(&settings.site_url);
let cookie = self.variables.interpolate(&settings.cookie_datadome);
let ua = self.variables.interpolate(&settings.user_agent);
let custom_wasm = if !settings.custom_wasm_b64.is_empty() {
    Some(base64::decode(&settings.custom_wasm_b64)?)
} else { None };
// Generate DataDome interstitial sensor payload
let sensor = datadome::generate_sensor(&site_url, &cookie, &ua, custom_wasm.as_deref())?;
self.variables.set_user(&settings.output_var, sensor, settings.capture);`,
	},
	{
		type: 'RandomData',
		name: 'Random Data',
		category: 'Utilities',
		description: 'Generates random data: strings, UUIDs, numbers, emails, names, addresses, phone numbers, and dates.',
		parameters: [
			{ name: 'data_type', type: 'enum', required: true, description: 'Type: String, Uuid, Number, Email, FirstName, LastName, FullName, StreetAddress, City, State, ZipCode, PhoneNumber, Date' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'RANDOM' },
			{ name: 'string_length', type: 'number', required: false, description: 'Length for random strings', default: '16' },
			{ name: 'string_charset', type: 'string', required: false, description: 'Character set: alphanumeric, alpha, numeric, hex, custom', default: 'alphanumeric' },
			{ name: 'number_min', type: 'number', required: false, description: 'Minimum for random numbers', default: '0' },
			{ name: 'number_max', type: 'number', required: false, description: 'Maximum for random numbers', default: '100' },
		],
		codeExample: `Type: Email
→ Generates a random email like john.doe42@example.com

Type: String, Length: 32, Charset: hex
→ Generates a random 32-char hex string`,
		tips: [
			'Email, names, and addresses generate realistic fake data',
			'Custom charset: set string_charset to "custom" and specify chars',
			'UUID generates a random v4 UUID',
		],
		relatedBlocks: ['SetVariable', 'StringFunction'],
		rustCode: `let value = match settings.data_type {
    RandomDataType::String => random_string(settings.string_length, &settings.string_charset),
    RandomDataType::Uuid => Uuid::new_v4().to_string(),
    RandomDataType::Number => random_number(settings.number_min, settings.number_max, settings.number_decimal),
    RandomDataType::Email => format!("{}@{}.com", random_word(), random_word()),
    RandomDataType::FirstName => FIRST_NAMES.choose(&mut rng).unwrap().to_string(),
    RandomDataType::FullName => format!("{} {}", random_first_name(), random_last_name()),
    RandomDataType::PhoneNumber => format!("+1{}", random_digits(10)),
    RandomDataType::Date => random_date(&settings.date_format, &settings.date_min, &settings.date_max),
    // ... City, State, ZipCode, StreetAddress, LastName
};
self.variables.set_user(&settings.output_var, value, settings.capture);`,
	},
	{
		type: 'Plugin',
		name: 'Plugin Block',
		category: 'Utilities',
		description: 'Executes a block provided by an external plugin DLL. Plugin blocks extend reqflow with custom functionality.',
		parameters: [
			{ name: 'plugin_block_type', type: 'string', required: true, description: 'Block type identifier (PluginName.BlockName)' },
			{ name: 'settings_json', type: 'string', required: false, description: 'JSON settings for the plugin block', default: '{}' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'PLUGIN_RESULT' },
		],
		codeExample: `Plugin block type: ExamplePlugin.ReverseString
Settings: {"input_var": "data.SOURCE"}
→ Executes the plugin block with the given settings`,
		tips: [
			'Import plugins via File → Import Plugin (.dll)',
			'Plugin settings are rendered as a form when schema is available',
			'Plugin blocks appear in the palette under their declared category',
		],
		relatedBlocks: ['Script', 'SetVariable'],
		rustCode: `let pm = self.plugin_manager.as_ref()?;
let settings_json = self.variables.interpolate(&settings.settings_json);
let vars_snapshot = serde_json::to_string(&self.variables.snapshot())?;

// Call plugin DLL via FFI (plugin_execute export)
let (success, updated_vars, log) = pm.execute_block(
    &settings.plugin_block_type,
    &settings_json,
    &vars_snapshot,
)?;

// Apply variable updates from plugin
for (k, v) in updated_vars {
    self.variables.set_user(&k, v, settings.capture);
}`,
	},
	{
		type: 'Group',
		name: 'Group',
		category: 'Control',
		description: 'Organizational container that groups related blocks together. Child blocks execute sequentially when the group runs. Groups can be collapsed in the editor to reduce visual clutter.',
		parameters: [
			{ name: 'blocks', type: 'Block[]', required: false, description: 'Child blocks inside the group', default: '[]' },
			{ name: 'collapsed', type: 'boolean', required: false, description: 'Whether the group is visually collapsed', default: 'false' },
		],
		codeExample: `Group "Auth Flow"
  ├─ HTTP Request (login)
  ├─ Parse JSON (token)
  └─ Key Check (status)

Drag blocks into the group container to organize your pipeline.`,
		tips: [
			'Use groups to organize complex pipelines into logical sections',
			'Toggle collapse in settings to hide/show group contents',
			'Drag blocks from the palette directly into a group',
			'Groups execute their children sequentially, just like the main pipeline',
		],
		relatedBlocks: ['IfElse', 'Loop'],
		rustCode: `// Group simply executes its child blocks sequentially
self.execute_blocks(&settings.blocks, sidecar_tx).await`,
	},
];

export const GUIDE_SECTIONS: GuideSection[] = [
	{
		id: 'get-started',
		title: 'Get Started',
		icon: 'Rocket',
		content: `
<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">What is reqflow?</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:16px">
reqflow is a pipeline-based HTTP automation engine. You build <strong>configs</strong> — sequences of blocks that describe an HTTP workflow — then run them at scale against a list of inputs (wordlists). Each input line is parsed into variables like <code>&lt;USER&gt;</code> and <code>&lt;PASS&gt;</code>, then flows through the pipeline.
</p>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Core Concepts</h3>
<ul style="font-size:12px;line-height:1.8;margin-bottom:16px;padding-left:20px">
<li><strong>Pipeline</strong> — An ordered list of blocks that run sequentially for each input line</li>
<li><strong>Variables</strong> — Data flows between blocks via named variables. Use <code>&lt;VAR&gt;</code> syntax to interpolate. Built-in: <code>&lt;USER&gt;</code>, <code>&lt;PASS&gt;</code>, <code>&lt;data.SOURCE&gt;</code>, <code>&lt;data.RESPONSECODE&gt;</code></li>
<li><strong>Bot Status</strong> — Each line ends with a status: <em>SUCCESS</em>, <em>FAIL</em>, <em>BAN</em>, <em>RETRY</em>, <em>CUSTOM</em>, or <em>NONE</em>. Set by KeyCheck blocks</li>
<li><strong>Captures</strong> — Variables marked as "capture" are saved with hits (e.g., account balance, email, token)</li>
</ul>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Basic Workflow</h3>
<ol style="font-size:12px;line-height:1.8;margin-bottom:16px;padding-left:20px">
<li><strong>Create a config</strong> — File → New Config, or open an existing <code>.yaml</code></li>
<li><strong>Add blocks</strong> — Drag blocks from the palette: typically HttpRequest → Parser → KeyCheck</li>
<li><strong>Configure blocks</strong> — Click a block to edit its settings (URL, headers, parse rules, check conditions)</li>
<li><strong>Load data</strong> — Import a wordlist file with one entry per line (<code>user:pass</code> format)</li>
<li><strong>Run</strong> — Set thread count and proxy list, click Start</li>
</ol>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">First Pipeline Example</h3>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
1. HttpRequest
   URL: https://example.com/api/login
   Method: POST
   Body: username=&lt;USER&gt;&amp;password=&lt;PASS&gt;

2. ParseJSON
   JSONPath: $.token
   Output var: TOKEN

3. KeyCheck
   SUCCESS when data.RESPONSECODE EqualTo "200"
   FAIL (default)
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Variable Interpolation</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:8px">
All string fields in blocks support <code>&lt;VARIABLE_NAME&gt;</code> interpolation:
</p>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
&lt;input.EMAIL&gt;       — Input line fields
&lt;data.SOURCE&gt;       — Response body
&lt;data.RESPONSECODE&gt; — HTTP status code
&lt;TOKEN&gt;             — User-defined variable
&lt;random.uuid&gt;       — Random UUID
&lt;random.email&gt;      — Random email
&lt;random.string.32&gt;  — Random 32-char string
&lt;random.number.1.100&gt; — Random number 1-100
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Keyboard Shortcuts</h3>
<table style="font-size:11px;width:100%;border-collapse:collapse;margin-bottom:16px">
<tr style="border-bottom:1px solid var(--border)"><td style="padding:4px 8px"><kbd>Ctrl+S</kbd></td><td style="padding:4px 8px">Save config</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:4px 8px"><kbd>F1</kbd></td><td style="padding:4px 8px">Open docs</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:4px 8px"><kbd>F5</kbd></td><td style="padding:4px 8px">Debug current line</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:4px 8px"><kbd>Ctrl+D</kbd></td><td style="padding:4px 8px">Duplicate block</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:4px 8px"><kbd>Delete</kbd></td><td style="padding:4px 8px">Remove selected block</td></tr>
<tr><td style="padding:4px 8px"><kbd>Ctrl+Z / Y</kbd></td><td style="padding:4px 8px">Undo / Redo</td></tr>
</table>
`,
	},
	{
		id: 'plugin-kit',
		title: 'Plugin Kit',
		icon: 'Puzzle',
		content: `
<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Creating a Plugin</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:16px">
Plugins are Rust shared libraries (<code>.dll</code> / <code>.so</code>) that extend reqflow with custom block types. A plugin exports a set of C-ABI functions that the engine calls to discover and execute blocks.
</p>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">1. Cargo.toml Setup</h3>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]   # Required: builds a C-compatible shared library

[dependencies]
serde_json = "1"
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">2. ABI Structs</h3>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
#[repr(C)]
pub struct PluginInfo {
    pub name: *const c_char,        // "My Plugin"
    pub version: *const c_char,     // "1.0.0"
    pub description: *const c_char, // "Does something useful"
    pub block_count: u32,           // Number of blocks provided
}

#[repr(C)]
pub struct BlockInfo {
    pub type_name: *const c_char,   // "MyPlugin.MyBlock"
    pub label: *const c_char,       // "My Block"
    pub category: *const c_char,    // "Utilities"
    pub color: *const c_char,       // "#4ec9b0"
    pub settings_schema: *const c_char, // JSON Schema for settings UI
}

#[repr(C)]
pub struct ExecuteResult {
    pub success: bool,
    pub updated_variables_json: *const c_char,
    pub log_message: *const c_char,
    pub error_message: *const c_char,
}
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">3. Required Exports</h3>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
#[no_mangle]
pub extern "C" fn plugin_info() -> *mut PluginInfo { ... }

#[no_mangle]
pub extern "C" fn plugin_block_info(index: u32) -> *mut BlockInfo { ... }

#[no_mangle]
pub extern "C" fn plugin_execute(
    block_type: *const c_char,    // Which block to run
    settings_json: *const c_char, // Block settings as JSON
    variables_json: *const c_char // Current variables as JSON
) -> *mut ExecuteResult { ... }

#[no_mangle]
pub extern "C" fn plugin_free_string(ptr: *mut c_char) { ... }
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">4. Execute Example</h3>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
#[no_mangle]
pub extern "C" fn plugin_execute(
    _block_type: *const c_char,
    settings_json: *const c_char,
    variables_json: *const c_char,
) -> *mut ExecuteResult {
    let settings: HashMap&lt;String, String&gt; = serde_json::from_str(&settings_str).unwrap_or_default();
    let mut vars: HashMap&lt;String, String&gt; = serde_json::from_str(&vars_str).unwrap_or_default();

    // Your custom logic here
    let input = vars.get("data.SOURCE").cloned().unwrap_or_default();
    let reversed: String = input.chars().rev().collect();
    vars.insert("PLUGIN_RESULT".to_string(), reversed);

    Box::into_raw(Box::new(ExecuteResult {
        success: true,
        updated_variables_json: CString::new(serde_json::to_string(&vars).unwrap()).unwrap().into_raw(),
        log_message: CString::new("Processed OK").unwrap().into_raw(),
        error_message: std::ptr::null(),
    }))
}
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">5. Settings JSON Schema</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:8px">
Return a JSON Schema string from <code>BlockInfo.settings_schema</code> to get auto-generated settings UI:
</p>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
{
  "type": "object",
  "properties": {
    "input_var": { "type": "string", "default": "data.SOURCE", "title": "Input Variable" },
    "mode": { "type": "string", "enum": ["fast", "accurate"], "default": "fast", "title": "Mode" }
  }
}
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">6. Build &amp; Load</h3>
<ol style="font-size:12px;line-height:1.8;padding-left:20px">
<li><code>cargo build --release</code> → produces <code>target/release/my_plugin.dll</code></li>
<li>In reqflow GUI: File → Import Plugin → select the <code>.dll</code></li>
<li>Plugin blocks appear in the block palette under their declared category</li>
</ol>
`,
	},
	{
		id: 'runners',
		title: 'Runners',
		icon: 'Play',
		content: `
<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Architecture</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:16px">
The runner system is an orchestrator that manages concurrent workers processing a shared data pool. Each worker picks a line from the pool, parses it into input variables, executes the full pipeline, and reports the result.
</p>

<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
┌─────────────────────────────────────┐
│           Orchestrator              │
│  ┌──────────┐  ┌──────────────────┐ │
│  │ Data Pool │  │   Proxy Pool    │ │
│  │ (lines)   │  │ (round-robin)   │ │
│  └─────┬─────┘  └────────┬────────┘ │
│        │                 │          │
│  ┌─────▼─────────────────▼────────┐ │
│  │     Worker 1  │  Worker 2  ... │ │
│  │  get line     │  get line      │ │
│  │  get proxy    │  get proxy     │ │
│  │  run pipeline │  run pipeline  │ │
│  │  report stats │  report stats  │ │
│  └────────────────────────────────┘ │
└─────────────────────────────────────┘
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Execution Flow (per line)</h3>
<ol style="font-size:12px;line-height:1.8;margin-bottom:16px;padding-left:20px">
<li><strong>Get line</strong> — Worker pulls the next unprocessed line from the data pool</li>
<li><strong>Parse input</strong> — Line is split by the delimiter (default <code>:</code>) into <code>&lt;USER&gt;</code>, <code>&lt;PASS&gt;</code>, etc.</li>
<li><strong>Get proxy</strong> — Round-robin selection from the proxy pool (if proxies loaded)</li>
<li><strong>Execute pipeline</strong> — Each block runs in sequence, passing variables forward</li>
<li><strong>Check status</strong> — Final bot status determines the outcome</li>
<li><strong>Update stats</strong> — Increment counters (hits, fails, bans, retries, tested, CPM)</li>
</ol>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Bot Statuses</h3>
<table style="font-size:11px;width:100%;border-collapse:collapse;margin-bottom:16px">
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600;color:#22c55e">SUCCESS</td><td style="padding:6px 8px">Valid credentials — saved to hits file with captures</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600;color:#ef4444">FAIL</td><td style="padding:6px 8px">Invalid credentials — line is discarded</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600;color:#f59e0b">BAN</td><td style="padding:6px 8px">IP/account blocked — proxy is temp-banned, line retried with different proxy</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600;color:#3b82f6">RETRY</td><td style="padding:6px 8px">Temporary error — line goes back to pool for retry</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600;color:#a855f7">CUSTOM</td><td style="padding:6px 8px">User-defined status — saved separately (e.g., "2FA", "Locked")</td></tr>
<tr><td style="padding:6px 8px;font-weight:600;color:#6b7280">NONE</td><td style="padding:6px 8px">No KeyCheck matched — treated as fail</td></tr>
</table>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Retry Logic</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:8px">
When a line gets RETRY or BAN status, or when a network error occurs:
</p>
<ul style="font-size:12px;line-height:1.8;padding-left:20px;margin-bottom:16px">
<li>Line is placed back in the data pool</li>
<li>For BAN: the proxy is temporarily removed from rotation</li>
<li>Max retry count is configurable (default: 3) — after that, marked as TOCHECK</li>
<li>Retried lines get a fresh proxy assignment</li>
</ul>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Stats Tracking</h3>
<p style="font-size:12px;line-height:1.7">
The runner tracks: <strong>Hits</strong> (success count), <strong>Fails</strong>, <strong>Bans</strong>, <strong>Retries</strong>, <strong>Tested</strong> (total processed), <strong>CPM</strong> (checks per minute), <strong>Progress</strong> (tested / total), and <strong>Elapsed</strong> time. Stats update in real-time in the GUI status bar.
</p>
`,
	},
	{
		id: 'proxies',
		title: 'Proxies',
		icon: 'Shield',
		content: `
<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Proxy Formats</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:8px">
reqflow accepts proxy lists in multiple formats, one proxy per line:
</p>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
# Format 1: HOST:PORT (defaults to HTTP)
192.168.1.1:8080

# Format 2: Protocol URL
http://192.168.1.1:8080
https://proxy.example.com:3128
socks5://192.168.1.1:1080

# Format 3: Protocol URL with auth
socks5://user:pass@192.168.1.1:1080

# Format 4: TYPE:HOST:PORT:USER:PASS
http:192.168.1.1:8080:username:password
socks5:10.0.0.1:1080:user:pass
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Supported Proxy Types</h3>
<table style="font-size:11px;width:100%;border-collapse:collapse;margin-bottom:16px">
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600">HTTP</td><td style="padding:6px 8px">Standard HTTP/1.1 proxy with CONNECT tunneling</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600">HTTPS</td><td style="padding:6px 8px">HTTP proxy over TLS connection</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600">SOCKS4</td><td style="padding:6px 8px">SOCKS4 protocol (no authentication support)</td></tr>
<tr><td style="padding:6px 8px;font-weight:600">SOCKS5</td><td style="padding:6px 8px">SOCKS5 protocol with optional username/password auth</td></tr>
</table>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Rotation &amp; Banning</h3>
<ul style="font-size:12px;line-height:1.8;padding-left:20px;margin-bottom:16px">
<li><strong>Round-robin</strong> — Proxies are assigned to workers in order, cycling through the list</li>
<li><strong>Temp ban</strong> — When a proxy gets a BAN result, it's removed from rotation for a configurable duration (default: 30s)</li>
<li><strong>Auto-recovery</strong> — Banned proxies are automatically re-added after the ban period expires</li>
<li><strong>No proxies</strong> — If no proxy list is loaded, all requests go through the direct connection</li>
</ul>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Best Practices</h3>
<ul style="font-size:12px;line-height:1.8;padding-left:20px">
<li>Use <strong>residential proxies</strong> for sites with strong anti-bot protection</li>
<li>Keep the thread count proportional to your proxy count (1-3 threads per proxy)</li>
<li>Test proxies before a run — dead proxies waste time and cause retries</li>
<li>Use SOCKS5 for sites that block datacenter HTTP proxies</li>
<li>Match your User-Agent TLS fingerprint to the proxy type for consistency</li>
</ul>
`,
	},
];
