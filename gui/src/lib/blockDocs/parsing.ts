import type { BlockDoc } from './types';

export const PARSING_DOCS: BlockDoc[] = [
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
];
