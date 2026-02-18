import type { BlockDoc } from './types';

export const CONTROL_DOCS: BlockDoc[] = [
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
		description: 'A flexible code block for custom logic that native blocks cannot express. Write freeform code to manipulate variables, perform lookups, convert data, unescape text, or interact with the filesystem. Script blocks are also the target for imported SVB/LoliCode constructs that have no direct block equivalent. When exporting to Rust via Code View, recognized patterns are compiled to real Rust code automatically.',
		parameters: [
			{ name: 'code', type: 'string', required: true, description: 'Script code. Supports freeform logic or structured SVB patterns (Translate, UnixTimeToDate, Unescape, Split, Utility). Use <VAR> syntax to reference pipeline variables.' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'RESULT' },
			{ name: 'capture', type: 'boolean', required: false, description: 'Mark the output variable as captured (included in hit output)', default: 'false' },
		],
		codeExample: `── Lookup Table (Translate) ──
Maps an input variable to a value via key→value pairs.
Imported from SVB FUNCTION Translate blocks.

  Input: country1 = "US"
  Table: "US" => "1", "GB" => "44", "DE" => "49", ...
  Output: Code = "1"

  → Generates: match country1.as_str() {
       "US" => "1".to_string(),
       "GB" => "44".to_string(),
       _ => String::new(),
     }

── Unix Timestamp to Date ──
Converts a unix timestamp to a formatted date string.

  Input: memberSince = "1609459200"
  Format: "yyyy-MM-dd"
  Output: memberSince = "2021-01-01"

  → Generates: chrono::DateTime::from_timestamp(ts, 0)
       .map(|dt| dt.format("%Y-%m-%d").to_string())

── HTML Unescape ──
Decodes HTML entities (&amp; &lt; &gt; &quot; &#39;)
and numeric entities (&#NNN; &#xHHH;).

  Input: planPrice = "9&period;99 &amp; tax"
  Output: planPrice = "9.99 & tax"

── Split ──
Splits a string by a separator and takes an element by index.

  Input: data = "a,b,c"
  Separator: ","   Index: 1
  Output: result = "b"

  → Generates: data.split(",").nth(1).unwrap_or("")

── File Utility (AppendLines) ──
Appends interpolated text to a file. Creates parent
directories if needed.

  File: "Hits/output.txt"
  Content: "Credentials: <USER>:<PASS>\\nPlan: <PlanName>"
  → Appends formatted text with variables interpolated`,
		tips: [
			'Script blocks are created automatically when importing SVB configs with FUNCTION or UTILITY commands',
			'Recognized patterns (Translate, UnixTimeToDate, Unescape, Split, File Utility) generate real Rust in Code View',
			'Unrecognized scripts export as commented-out TODO stubs — implement them manually in the generated code',
			'For simple variable manipulation, prefer SetVariable or StringFunction blocks instead',
			'Date format tokens are auto-converted: yyyy→%Y, MM→%m, dd→%d, HH→%H, mm→%M, ss→%S',
			'Translate tables support hundreds of entries — the generated match statement is optimized by the Rust compiler',
		],
		relatedBlocks: ['SetVariable', 'IfElse', 'StringFunction', 'ConversionFunction', 'DateFunction'],
		rustCode: `// ── Translate (lookup table) ──
let code = match country1.as_str() {
    "US" => "1".to_string(),
    "GB" => "44".to_string(),
    "DE" => "49".to_string(),
    _ => String::new(),
};

// ── UnixTimeToDate ──
let member_since = {
    let ts: i64 = member_since.parse().unwrap_or(0);
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
};

// ── Unescape (HTML entities) ──
let plan_price = {
    let mut s = plan_price.clone();
    s = s.replace("&amp;", "&");
    s = s.replace("&lt;", "<");
    s = s.replace("&gt;", ">");
    s = s.replace("&quot;", "\\"");
    s = s.replace("&#39;", "'");
    // Also decodes &#NNN; and &#xHHH; numeric entities
    s
};

// ── Split ──
let result = data.split(",").nth(1)
    .unwrap_or("").to_string();

// ── File Utility (AppendLines) ──
{
    let path = std::path::Path::new("Hits/output.txt");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true).append(true).open(path)?;
    writeln!(f, "{}:{}", user, pass)?;
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
