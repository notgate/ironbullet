/**
 * IronBullet Intellisense engine
 *
 * Builds context-aware suggestion lists for VariableInput fields and
 * the Response Viewer LR delimiter inputs. No external dependencies.
 */

import type { SuggestionItem } from './components/Intellisense.svelte';
import type { Pipeline } from './types';

// ── Static suggestion banks ────────────────────────────────────────────────

/** Always-available data namespace variables */
const DATA_VARS: SuggestionItem[] = [
	{ label: 'data.SOURCE',           insertText: '<data.SOURCE>',           kind: 'data',     detail: 'string', documentation: 'Response body of the last HTTP request.' },
	{ label: 'data.RESPONSECODE',     insertText: '<data.RESPONSECODE>',     kind: 'data',     detail: 'int',    documentation: 'HTTP status code of the last response.' },
	{ label: 'data.ADDRESS',          insertText: '<data.ADDRESS>',          kind: 'data',     detail: 'string', documentation: 'Final URL after redirects.' },
	{ label: 'data.HEADERS',          insertText: '<data.HEADERS>',          kind: 'data',     detail: 'string', documentation: 'Response headers as a raw string.' },
	{ label: 'data.COOKIES',          insertText: '<data.COOKIES>',          kind: 'data',     detail: 'string', documentation: 'Response cookies as name=value; pairs.' },
	{ label: 'data.ERROR',            insertText: '<data.ERROR>',            kind: 'data',     detail: 'string', documentation: 'Error message from the last failed block.' },
	{ label: 'data.STATUS',           insertText: '<data.STATUS>',           kind: 'data',     detail: 'string', documentation: 'Current checker status: Success, Fail, Ban, etc.' },
];

/** Always-available input variables */
const INPUT_VARS: SuggestionItem[] = [
	{ label: 'input.USER',  insertText: '<input.USER>',  kind: 'input', detail: 'string', documentation: 'Username part of the current wordlist line.' },
	{ label: 'input.PASS',  insertText: '<input.PASS>',  kind: 'input', detail: 'string', documentation: 'Password part of the current wordlist line.' },
	{ label: 'input.DATA',  insertText: '<input.DATA>',  kind: 'input', detail: 'string', documentation: 'Full unsplit wordlist line.' },
	{ label: '<USER>',      insertText: '<USER>',         kind: 'keyword', detail: 'embed', documentation: 'Shorthand embed for the username.' },
	{ label: '<PASS>',      insertText: '<PASS>',         kind: 'keyword', detail: 'embed', documentation: 'Shorthand embed for the password.' },
];

/** Common JSON path patterns for ParseJSON blocks */
const JSON_PATH_SNIPPETS: SuggestionItem[] = [
	{ label: 'token',            insertText: 'token',                     kind: 'snippet', detail: 'json path' },
	{ label: 'access_token',     insertText: 'access_token',             kind: 'snippet', detail: 'json path' },
	{ label: 'refresh_token',    insertText: 'refresh_token',            kind: 'snippet', detail: 'json path' },
	{ label: 'data.token',       insertText: 'data.token',               kind: 'snippet', detail: 'json path' },
	{ label: 'user.id',          insertText: 'user.id',                  kind: 'snippet', detail: 'json path' },
	{ label: 'user.email',       insertText: 'user.email',               kind: 'snippet', detail: 'json path' },
	{ label: 'message',          insertText: 'message',                  kind: 'snippet', detail: 'json path' },
	{ label: 'error',            insertText: 'error',                    kind: 'snippet', detail: 'json path' },
	{ label: 'success',          insertText: 'success',                  kind: 'snippet', detail: 'json path' },
	{ label: 'result',           insertText: 'result',                   kind: 'snippet', detail: 'json path' },
	{ label: 'balance',          insertText: 'balance',                  kind: 'snippet', detail: 'json path' },
	{ label: 'items[0]',         insertText: 'items[0]',                 kind: 'snippet', detail: 'json path' },
];

/** Common regex patterns */
const REGEX_SNIPPETS: SuggestionItem[] = [
	{ label: '(\\w+)',               insertText: '(\\w+)',              kind: 'snippet', detail: 'word' },
	{ label: '(\\d+)',               insertText: '(\\d+)',              kind: 'snippet', detail: 'digits' },
	{ label: '([^"]+)',              insertText: '([^"]+)',             kind: 'snippet', detail: 'non-quote' },
	{ label: '([a-zA-Z0-9+/=]+)',   insertText: '([a-zA-Z0-9+/=]+)', kind: 'snippet', detail: 'base64' },
	{ label: '([a-f0-9]{32})',      insertText: '([a-f0-9]{32})',     kind: 'snippet', detail: 'md5 hash' },
	{ label: '([a-f0-9]{64})',      insertText: '([a-f0-9]{64})',     kind: 'snippet', detail: 'sha256 hash' },
	{ label: '(https?://[^\\s]+)',  insertText: '(https?://[^\\s]+)', kind: 'snippet', detail: 'url' },
	{ label: '([\\w.-]+@[\\w.-]+)', insertText: '([\\w.-]+@[\\w.-]+)', kind: 'snippet', detail: 'email' },
];

/** Common CSS attribute names */
const CSS_ATTR_SNIPPETS: SuggestionItem[] = [
	{ label: 'innerText', insertText: 'innerText', kind: 'snippet', detail: 'default' },
	{ label: 'href',      insertText: 'href',       kind: 'snippet', detail: 'link URL' },
	{ label: 'src',       insertText: 'src',        kind: 'snippet', detail: 'image/script src' },
	{ label: 'value',     insertText: 'value',      kind: 'snippet', detail: 'input value' },
	{ label: 'data-*',    insertText: 'data-',      kind: 'snippet', detail: 'data attribute' },
	{ label: 'class',     insertText: 'class',      kind: 'snippet', detail: 'class names' },
	{ label: 'id',        insertText: 'id',         kind: 'snippet', detail: 'element id' },
	{ label: 'name',      insertText: 'name',       kind: 'snippet', detail: 'input name' },
	{ label: 'action',    insertText: 'action',     kind: 'snippet', detail: 'form action' },
	{ label: 'content',   insertText: 'content',    kind: 'snippet', detail: 'meta content' },
];

/** Common LR left delimiters seen in real-world APIs */
const LR_LEFT_SNIPPETS: SuggestionItem[] = [
	{ label: '"token":"',          insertText: '"token":"',           kind: 'ldelim', detail: 'JSON token' },
	{ label: '"access_token":"',   insertText: '"access_token":"',   kind: 'ldelim', detail: 'JSON oauth' },
	{ label: '"value":"',          insertText: '"value":"',           kind: 'ldelim', detail: 'JSON value' },
	{ label: '"id":',              insertText: '"id":',               kind: 'ldelim', detail: 'JSON id' },
	{ label: '"message":"',        insertText: '"message":"',         kind: 'ldelim', detail: 'JSON msg' },
	{ label: '"data":"',           insertText: '"data":"',            kind: 'ldelim', detail: 'JSON data' },
	{ label: 'name="csrf" value="',insertText: 'name="csrf" value="',kind: 'ldelim', detail: 'CSRF form' },
	{ label: 'name="_token" value="', insertText: 'name="_token" value="', kind: 'ldelim', detail: 'Laravel CSRF' },
	{ label: 'window.__token = "', insertText: 'window.__token = "', kind: 'ldelim', detail: 'JS inline token' },
	{ label: 'Set-Cookie: ',       insertText: 'Set-Cookie: ',       kind: 'ldelim', detail: 'cookie header' },
	{ label: 'Location: ',         insertText: 'Location: ',         kind: 'ldelim', detail: 'redirect url' },
];

/** Common LR right delimiters */
const LR_RIGHT_SNIPPETS: SuggestionItem[] = [
	{ label: '"',   insertText: '"',   kind: 'rdelim', detail: 'close quote' },
	{ label: '",',  insertText: '",',  kind: 'rdelim', detail: 'JSON string end' },
	{ label: '}',   insertText: '}',   kind: 'rdelim', detail: 'close brace' },
	{ label: "'",   insertText: "'",   kind: 'rdelim', detail: 'single quote' },
	{ label: '\\r', insertText: '\\r', kind: 'rdelim', detail: 'carriage return' },
	{ label: '\\n', insertText: '\\n', kind: 'rdelim', detail: 'newline' },
	{ label: ';',   insertText: ';',   kind: 'rdelim', detail: 'semicolon' },
	{ label: '&',   insertText: '&',   kind: 'rdelim', detail: 'query param end' },
	{ label: '</',  insertText: '</',  kind: 'rdelim', detail: 'HTML close tag' },
];

/** Lambda expression snippets */
const LAMBDA_SNIPPETS: SuggestionItem[] = [
	{ label: "x => x.split(',')[0]",        insertText: "x => x.split(',')[0]",       kind: 'snippet', detail: 'split first' },
	{ label: "x => x.split(':')[1]",        insertText: "x => x.split(':')[1]",       kind: 'snippet', detail: 'split second' },
	{ label: 'x => x.trim()',               insertText: 'x => x.trim()',              kind: 'snippet', detail: 'trim whitespace' },
	{ label: 'x => x.toUpperCase()',        insertText: 'x => x.toUpperCase()',       kind: 'snippet', detail: 'uppercase' },
	{ label: 'x => x.toLowerCase()',        insertText: 'x => x.toLowerCase()',       kind: 'snippet', detail: 'lowercase' },
	{ label: "x => x.replace('a', 'b')",   insertText: "x => x.replace('a', 'b')",  kind: 'snippet', detail: 'replace' },
	{ label: 'x => x.substring(0, 8)',      insertText: 'x => x.substring(0, 8)',     kind: 'snippet', detail: 'substring' },
	{ label: 'x => x.indexOf("=")',         insertText: 'x => x.indexOf("=")',        kind: 'snippet', detail: 'find index' },
];

/** URL / host suggestions for HTTP blocks */
const URL_SNIPPETS: SuggestionItem[] = [
	{ label: 'https://<HOST>/api/',         insertText: 'https://<HOST>/api/',        kind: 'snippet', detail: 'api base' },
	{ label: 'https://<HOST>/login',        insertText: 'https://<HOST>/login',       kind: 'snippet', detail: 'login' },
	{ label: 'https://<HOST>/auth/token',   insertText: 'https://<HOST>/auth/token', kind: 'snippet', detail: 'oauth token' },
	{ label: 'https://<HOST>/graphql',      insertText: 'https://<HOST>/graphql',    kind: 'snippet', detail: 'graphql' },
];

/** KeyCheck condition value suggestions */
const KEYCHECK_VALUE_SNIPPETS: SuggestionItem[] = [
	{ label: '200', insertText: '200', kind: 'snippet', detail: 'HTTP OK' },
	{ label: '401', insertText: '401', kind: 'snippet', detail: 'Unauthorized' },
	{ label: '403', insertText: '403', kind: 'snippet', detail: 'Forbidden' },
	{ label: '429', insertText: '429', kind: 'snippet', detail: 'Rate Limited' },
	{ label: '302', insertText: '302', kind: 'snippet', detail: 'Redirect' },
	{ label: 'true',  insertText: 'true',  kind: 'keyword', detail: 'bool' },
	{ label: 'false', insertText: 'false', kind: 'keyword', detail: 'bool' },
];

// ── Pipeline variable extraction ───────────────────────────────────────────

function extractPipelineVars(pipeline: Pipeline): SuggestionItem[] {
	const vars = new Map<string, SuggestionItem>();

	function walk(blocks: any[]) {
		for (const block of blocks) {
			const s = block.settings ?? {};

			// output_var / response_var define new variables
			for (const key of ['output_var', 'response_var']) {
				const v = s[key];
				if (v && typeof v === 'string' && v.trim()) {
					vars.set(v, {
						label: v,
						insertText: `<${v}>`,
						kind: 'variable',
						detail: 'var',
						documentation: `Defined by ${s.type ?? 'block'} block.`,
					});
					// Also expose data.VAR since HTTP response_var becomes data.VAR
					if (key === 'response_var') {
						const dk = `data.${v}`;
						vars.set(dk, {
							label: dk,
							insertText: `<${dk}>`,
							kind: 'data',
							detail: 'response body',
						});
						vars.set(`${dk}.HEADERS`, {
							label: `${dk}.HEADERS`,
							insertText: `<${dk}.HEADERS>`,
							kind: 'data',
							detail: 'response headers',
						});
						vars.set(`${dk}.COOKIES`, {
							label: `${dk}.COOKIES`,
							insertText: `<${dk}.COOKIES>`,
							kind: 'data',
							detail: 'response cookies',
						});
						vars.set(`${dk}.RESPONSECODE`, {
							label: `${dk}.RESPONSECODE`,
							insertText: `<${dk}.RESPONSECODE>`,
							kind: 'data',
							detail: 'status code',
						});
					}
				}
			}

			// Constants block
			if (Array.isArray(s.constants)) {
				for (const c of s.constants) {
					if (c.name) {
						vars.set(c.name, {
							label: c.name,
							insertText: `<${c.name}>`,
							kind: 'variable',
							detail: 'constant',
							documentation: `Value: ${c.value ?? ''}`,
						});
					}
				}
			}

			// SetVariable block
			if (s.type === 'SetVariable' && s.name) {
				vars.set(s.name, {
					label: s.name,
					insertText: `<${s.name}>`,
					kind: 'variable',
					detail: 'set variable',
				});
			}

			// Loop item variable
			if (s.item_var) {
				vars.set(s.item_var, {
					label: s.item_var,
					insertText: `<${s.item_var}>`,
					kind: 'variable',
					detail: 'loop item',
				});
			}

			// ParseJSON outputs with CLAIM_ prefix (JwtToken)
			if (s.type === 'JwtToken' && s.output_var) {
				vars.set(`CLAIM_`, {
					label: 'CLAIM_*',
					insertText: 'CLAIM_',
					kind: 'variable',
					detail: 'JWT claim',
					documentation: 'JWT decode writes CLAIM_<field> for each payload key.',
				});
			}

			// Recurse
			for (const key of ['true_blocks', 'false_blocks', 'blocks', 'startup_blocks']) {
				if (Array.isArray(s[key])) walk(s[key]);
			}
		}
	}

	walk(pipeline?.blocks ?? []);
	walk(pipeline?.startup_blocks ?? []);
	return Array.from(vars.values());
}

// ── Context-aware suggestion builder ─────────────────────────────────────

export type FieldContext =
	| 'variable'       // generic variable/embed field (VariableInput default)
	| 'input_var'      // "Input variable" field — data.* + pipeline vars
	| 'output_var'     // "Output var" field — free name (no suggestions, just current vars)
	| 'json_path'      // ParseJSON json_path
	| 'regex_pattern'  // ParseRegex pattern
	| 'css_selector'   // ParseCSS selector
	| 'css_attribute'  // ParseCSS attribute
	| 'lambda'         // LambdaParser expression
	| 'url'            // HTTP request URL
	| 'keycheck_value' // KeyCheck condition value
	| 'ldelim'         // LR left delimiter
	| 'rdelim'         // LR right delimiter
	| 'generic';       // everything else

/**
 * Build a filtered suggestion list for a given field context and query.
 *
 * @param ctx     - what kind of field this is
 * @param query   - current word being typed (used for prefix filter)
 * @param pipeline - current pipeline (for dynamic variable extraction)
 * @param responseBody - current response viewer body (for LR suggestions)
 */
export function buildSuggestions(
	ctx: FieldContext,
	query: string,
	pipeline: Pipeline | null,
	responseBody?: string,
): SuggestionItem[] {
	const pipelineVars = pipeline ? extractPipelineVars(pipeline) : [];
	const q = query.toLowerCase();

	let candidates: SuggestionItem[] = [];

	switch (ctx) {
		case 'input_var':
			candidates = [...DATA_VARS, ...INPUT_VARS, ...pipelineVars];
			break;

		case 'output_var':
			// Suggest existing vars so user can overwrite intentionally
			candidates = [...pipelineVars, ...DATA_VARS];
			break;

		case 'json_path':
			candidates = [...JSON_PATH_SNIPPETS, ...pipelineVars.filter(v => v.kind === 'variable')];
			break;

		case 'regex_pattern':
			candidates = REGEX_SNIPPETS;
			break;

		case 'css_attribute':
			candidates = CSS_ATTR_SNIPPETS;
			break;

		case 'lambda':
			candidates = LAMBDA_SNIPPETS;
			break;

		case 'url':
			candidates = [...URL_SNIPPETS, ...INPUT_VARS, ...DATA_VARS, ...pipelineVars];
			break;

		case 'keycheck_value':
			candidates = [...KEYCHECK_VALUE_SNIPPETS, ...DATA_VARS, ...pipelineVars];
			break;

		case 'ldelim':
			candidates = buildLRSuggestions('left', responseBody);
			break;

		case 'rdelim':
			candidates = buildLRSuggestions('right', responseBody);
			break;

		case 'variable':
		case 'generic':
		default:
			candidates = [...DATA_VARS, ...INPUT_VARS, ...pipelineVars];
			break;
	}

	// ── Response body next-word prediction ────────────────────────────────
	// When we have a real response body and the user has typed something,
	// prepend predictions derived from the actual response text.
	// This covers ldelim/rdelim and any other field when responseBody is supplied.
	let predictions: SuggestionItem[] = [];
	if (responseBody && query.length >= 2) {
		predictions = buildResponseBodyPredictions(query, responseBody);
	}

	// Filter static candidates by query
	const filtered = !q
		? candidates.slice(0, 16)
		: candidates.filter(s => s.label.toLowerCase().includes(q)).slice(0, 12);

	// Predictions come first (they're already filtered to contain the query as prefix)
	// then deduplicated static candidates
	const predSet = new Set(predictions.map(p => p.insertText));
	const deduped = filtered.filter(c => !predSet.has(c.insertText));

	return [...predictions, ...deduped].slice(0, 20);
}

/**
 * Extract candidate LR delimiters from response body.
 * Scans for JSON keys and HTML tag boundaries to suggest real delimiters.
 */
function buildLRSuggestions(side: 'left' | 'right', body?: string): SuggestionItem[] {
	const base = side === 'left' ? LR_LEFT_SNIPPETS : LR_RIGHT_SNIPPETS;
	if (!body) return base;

	const extras: SuggestionItem[] = [];

	if (side === 'left') {
		// Extract JSON string keys: "key": -> suggest '"key":"'
		const jsonKeyRe = /"([a-zA-Z_][a-zA-Z0-9_]{1,30})":/g;
		const seen = new Set<string>();
		let m: RegExpExecArray | null;
		while ((m = jsonKeyRe.exec(body)) !== null) {
			const delim = `"${m[1]}":"`;
			if (!seen.has(delim)) {
				seen.add(delim);
				extras.push({ label: delim, insertText: delim, kind: 'ldelim', detail: 'from response' });
				if (extras.length >= 10) break;
			}
		}
	} else {
		// For right side, extract chars that follow JSON string values
		const afterRe = /"([^"]{1,60})"([,}\];\n\r])/g;
		const seen = new Set<string>();
		let m: RegExpExecArray | null;
		while ((m = afterRe.exec(body)) !== null) {
			const ch = m[2];
			if (!seen.has(ch)) {
				seen.add(ch);
				extras.push({ label: ch === '\n' ? '\\n' : ch === '\r' ? '\\r' : ch, insertText: ch, kind: 'rdelim', detail: 'from response' });
				if (extras.length >= 6) break;
			}
		}
	}

	// Deduplicate against base
	const baseLabels = new Set(base.map(b => b.insertText));
	const uniqueExtras = extras.filter(e => !baseLabels.has(e.insertText));

	return [...uniqueExtras, ...base].slice(0, 20);
}

// ── Next-word / next-token prediction from response body ──────────────────

/**
 * Given the text the user has typed so far (query) and the full response body,
 * find every occurrence of query in the body and collect what comes immediately
 * after it — next word, next phrase up to a delimiter, and next individual char.
 *
 * Returns ranked SuggestionItems (most-frequent continuation first).
 *
 * Example: query = "Please" → body contains "Please check your" and "Please try again"
 *   → suggests "Please check", "Please check your", "Please try", "Please try again"
 */
export function buildResponseBodyPredictions(query: string, body: string): SuggestionItem[] {
	if (!body || query.length < 2) return [];

	// Score map: continuation string → count
	const scores = new Map<string, number>();

	// Scan every occurrence of the query in the body
	let searchFrom = 0;
	let safetyLimit = 0;
	while (safetyLimit++ < 500) {
		const idx = body.indexOf(query, searchFrom);
		if (idx === -1) break;
		searchFrom = idx + 1;

		const rest = body.slice(idx + query.length);
		if (!rest) continue;

		// Collect completions up to ~80 chars
		const segment = rest.slice(0, 80);

		// 1. Next single char (useful for delimiter hunting: what char follows this text?)
		const nextChar = segment[0];
		if (nextChar) {
			const k = query + nextChar;
			scores.set(k, (scores.get(k) ?? 0) + 1);
		}

		// 2. Next word boundary (space, quote, comma, bracket, newline stop the word)
		const nextWordMatch = segment.match(/^([\w\-\.@%+]+)/);
		if (nextWordMatch) {
			const k = query + nextWordMatch[1];
			scores.set(k, (scores.get(k) ?? 0) + 2); // weight word completions higher
		}

		// 3. Next phrase: consume words up to 6 tokens or a hard delimiter.
		// Emit cumulative prefixes so typing "Please" can suggest:
		//   "Please check", "Please check your", "Please check your email", etc.
		const phraseMatch = segment.match(/^([^\r\n"'<>{}\[\]]{1,80})/);
		if (phraseMatch && phraseMatch[1].trim().length > 0) {
			const phrase = phraseMatch[1];
			// Split on whitespace boundaries, preserving positions
			const tokenRe = /\S+/g;
			let tokenMatch: RegExpExecArray | null;
			let lastEnd = 0;
			let wordCount = 0;
			while ((tokenMatch = tokenRe.exec(phrase)) !== null && wordCount < 6) {
				lastEnd = tokenMatch.index + tokenMatch[0].length;
				const cumulative = query + phrase.slice(0, lastEnd);
				const score = Math.max(1, 6 - wordCount);
				scores.set(cumulative, (scores.get(cumulative) ?? 0) + score);
				wordCount++;
			}
		}
	}

	if (scores.size === 0) return [];

	// Sort by score descending, then by length ascending (shorter = more specific suggestion first)
	const sorted = Array.from(scores.entries())
		.filter(([k]) => k !== query) // exclude identical to query
		.sort(([a, sa], [b, sb]) => sb - sa || a.length - b.length)
		.slice(0, 12);

	return sorted.map(([text, count]) => ({
		label: text,
		insertText: text,
		kind: 'snippet' as const,
		detail: `×${count}`,
		documentation: 'Predicted from response body',
	}));
}

// ── Trigger detection ───────────────────────────────────────────────────────

/**
 * Returns the current "word" at cursor position that should trigger intellisense.
 * Triggers when:
 *   - user typed '<' (embed token start)
 *   - user typed a letter/dot and there's a partial word
 */
export function getQueryAtCursor(value: string, cursorPos: number): { query: string; triggerStart: number } | null {
	if (cursorPos === 0) return null;

	// Walk backwards to find word start
	let i = cursorPos - 1;
	while (i >= 0) {
		const ch = value[i];
		if (ch === '<') {
			// Embed token: trigger on <... pattern
			const query = value.slice(i + 1, cursorPos);
			return { query, triggerStart: i };
		}
		if (/[a-zA-Z0-9_.$]/.test(ch)) {
			i--;
		} else {
			break;
		}
	}

	const wordStart = i + 1;
	const word = value.slice(wordStart, cursorPos);

	// Only trigger if we have at least 1 char or started with '<'
	if (word.length >= 1) {
		return { query: word, triggerStart: wordStart };
	}

	return null;
}

/**
 * Apply a selected suggestion to an input value at the cursor.
 * Replaces the current word/token with the suggestion's insertText.
 */
export function applySuggestion(
	value: string,
	cursorPos: number,
	item: SuggestionItem,
): { newValue: string; newCursor: number } {
	const trigger = getQueryAtCursor(value, cursorPos);
	if (!trigger) {
		// Just insert at cursor
		const newValue = value.slice(0, cursorPos) + item.insertText + value.slice(cursorPos);
		return { newValue, newCursor: cursorPos + item.insertText.length };
	}

	const before = value.slice(0, trigger.triggerStart);
	const after = value.slice(cursorPos);

	// If we started with '<', check if there's a closing '>' to remove
	let suffix = after;
	if (value[trigger.triggerStart] === '<' && item.insertText.startsWith('<')) {
		// Replace up to next '>' if present
		const gtIdx = after.indexOf('>');
		if (gtIdx !== -1) suffix = after.slice(gtIdx + 1);
	}

	const newValue = before + item.insertText + suffix;
	const newCursor = before.length + item.insertText.length;
	return { newValue, newCursor };
}
