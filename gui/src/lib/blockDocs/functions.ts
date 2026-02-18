import type { BlockDoc } from './types';

export const FUNCTION_DOCS: BlockDoc[] = [
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
];
