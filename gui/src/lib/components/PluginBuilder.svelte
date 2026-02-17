<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import { toast } from '$lib/toast.svelte';
	import SkeuSelect from '$lib/components/SkeuSelect.svelte';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import Copy from '@lucide/svelte/icons/copy';
	import Download from '@lucide/svelte/icons/download';
	import Plus from '@lucide/svelte/icons/plus';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Puzzle from '@lucide/svelte/icons/puzzle';
	import Box from '@lucide/svelte/icons/box';
	import Zap from '@lucide/svelte/icons/zap';
	import Settings from '@lucide/svelte/icons/settings';
	import Package from '@lucide/svelte/icons/package';
	import Wrench from '@lucide/svelte/icons/wrench';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Play from '@lucide/svelte/icons/play';
	import Hammer from '@lucide/svelte/icons/hammer';
	import Bug from '@lucide/svelte/icons/bug';
	import FolderOpen from '@lucide/svelte/icons/folder-open';
	import CircleDot from '@lucide/svelte/icons/circle-dot';
	import Terminal from '@lucide/svelte/icons/terminal';
	import X from '@lucide/svelte/icons/x';
	import type * as Monaco from 'monaco-editor';

	// ── Plugin config state ──

	let pluginName = $state('MyPlugin');
	let pluginVersion = $state('0.1.0');
	let pluginAuthor = $state('');
	let pluginDescription = $state('A custom reqflow plugin');

	interface PluginBlock {
		name: string;
		label: string;
		category: string;
		color: string;
		settingsFields: Array<{ name: string; type: string; default: string }>;
	}

	let blocks = $state<PluginBlock[]>([{
		name: 'MyBlock',
		label: 'My Block',
		category: 'Utilities',
		color: '#9b59b6',
		settingsFields: [{ name: 'input_var', type: 'string', default: 'data.SOURCE' }],
	}]);

	let extraDeps = $state<Array<{ name: string; version: string; features: string }>>([]);

	// ── Sidebar navigation ──

	type Section = 'config' | 'getting-started' | 'abi-reference' | 'block-definition' | 'execution' | 'settings-schema' | 'dependencies' | 'building';
	let activeSection = $state<Section>('config');

	const SECTIONS: { id: Section; label: string; icon: typeof BookOpen }[] = [
		{ id: 'config', label: 'Plugin Config', icon: Settings },
		{ id: 'getting-started', label: 'Getting Started', icon: BookOpen },
		{ id: 'abi-reference', label: 'ABI Reference', icon: Box },
		{ id: 'block-definition', label: 'Block Definition', icon: Puzzle },
		{ id: 'execution', label: 'Execution', icon: Zap },
		{ id: 'settings-schema', label: 'Settings Schema', icon: Settings },
		{ id: 'dependencies', label: 'Dependencies', icon: Package },
		{ id: 'building', label: 'Build & Load', icon: Wrench },
	];

	const CATEGORIES = ['Requests', 'Parsing', 'Checks', 'Functions', 'Control', 'Utilities', 'Bypass', 'Sensors', 'Browser'];
	const FIELD_TYPES = [
		{ value: 'string', label: 'str' },
		{ value: 'number', label: 'num' },
		{ value: 'boolean', label: 'bool' },
	];

	// ── Build / output state ──

	let outputLines = $state<Array<{ text: string; type: 'info' | 'error' | 'success' | 'cmd' }>>([]);
	let isCompiling = $state(false);
	let showOutput = $state(false);
	let lastBuildSuccess = $state<boolean | null>(null);
	let lastDllPath = $state('');
	let projectDir = $state('');
	let buildRelease = $state(true);
	let outputEl: HTMLDivElement;

	// Breakpoints (line numbers in lib.rs where the user placed markers)
	let breakpoints = $state<Set<number>>(new Set());

	// ── Resizable panel state ──
	let centerPanelWidth = $state(360);
	let outputHeight = $state(180);

	function startResizeCenter(e: MouseEvent) {
		const startX = e.clientX;
		const startW = centerPanelWidth;
		const onMove = (ev: MouseEvent) => {
			centerPanelWidth = Math.max(260, Math.min(550, startW + ev.clientX - startX));
		};
		const onUp = () => {
			window.removeEventListener('mousemove', onMove);
			window.removeEventListener('mouseup', onUp);
		};
		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

	function startResizeOutput(e: MouseEvent) {
		const startY = e.clientY;
		const startH = outputHeight;
		const onMove = (ev: MouseEvent) => {
			outputHeight = Math.max(80, Math.min(400, startH - (ev.clientY - startY)));
		};
		const onUp = () => {
			window.removeEventListener('mousemove', onMove);
			window.removeEventListener('mouseup', onUp);
		};
		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

	function scrollOutputToBottom() {
		if (outputEl) {
			requestAnimationFrame(() => { outputEl.scrollTop = outputEl.scrollHeight; });
		}
	}

	function addOutput(text: string, type: 'info' | 'error' | 'success' | 'cmd' = 'info') {
		outputLines = [...outputLines, { text, type }];
		scrollOutputToBottom();
	}

	function clearOutput() {
		outputLines = [];
		lastBuildSuccess = null;
		lastDllPath = '';
	}

	function compilePlugin() {
		if (isCompiling) return;
		isCompiling = true;
		showOutput = true;
		addOutput('', 'cmd');
		addOutput('Starting build...', 'info');

		const libCode = editor?.getValue() || generateLibRs();
		const cargoCode = cargoEditor?.getValue() || generateCargoToml();

		app._compileOutputCallback = (data) => {
			if (data.line) {
				const isErr = data.line.startsWith('error') || data.line.includes('error[');
				const isWarn = data.line.startsWith('warning');
				const isSuc = data.line.startsWith('Build succeeded');
				addOutput(data.line, isSuc ? 'success' : isErr ? 'error' : isWarn ? 'error' : 'info');
			}
			if (data.done) {
				isCompiling = false;
				lastBuildSuccess = data.success;
				if (data.dll_path) lastDllPath = data.dll_path;
				addOutput(data.success ? 'Build completed successfully.' : 'Build failed.', data.success ? 'success' : 'error');
				app._compileOutputCallback = null;
			}
		};

		send('compile_plugin', {
			project_dir: projectDir,
			lib_rs: libCode,
			cargo_toml: cargoCode,
			release: buildRelease,
		});
	}

	function debugInspect() {
		showOutput = true;
		addOutput('', 'cmd');
		addOutput('── Plugin Debug Inspector ──', 'info');
		addOutput(`Plugin: ${pluginName} v${pluginVersion}`, 'info');
		addOutput(`Blocks: ${blocks.length}`, 'info');
		for (const b of blocks) {
			addOutput(`  [${b.category}] ${pluginName}.${b.name} — "${b.label}"`, 'info');
			addOutput(`    Color: ${b.color}  Fields: ${b.settingsFields.length}`, 'info');
			for (const f of b.settingsFields) {
				addOutput(`      ${f.name}: ${f.type} = "${f.default}"`, 'info');
			}
		}
		addOutput(`Extra deps: ${extraDeps.filter(d => d.name).length}`, 'info');
		for (const d of extraDeps.filter(d => d.name)) {
			addOutput(`  ${d.name} = "${d.version}"${d.features ? ` features=[${d.features}]` : ''}`, 'info');
		}
		if (breakpoints.size > 0) {
			addOutput(`Breakpoints set at lines: ${[...breakpoints].sort((a, b) => a - b).join(', ')}`, 'info');
		}
		// Validate
		const issues: string[] = [];
		if (!pluginName.trim()) issues.push('Plugin name is empty');
		for (const b of blocks) {
			if (!b.name.trim()) issues.push(`Block has empty name`);
			if (b.settingsFields.some(f => !f.name.trim())) issues.push(`Block "${b.name}" has field with empty name`);
			const dupes = b.settingsFields.map(f => f.name).filter((n, i, a) => a.indexOf(n) !== i);
			if (dupes.length) issues.push(`Block "${b.name}" has duplicate field names: ${dupes.join(', ')}`);
		}
		if (issues.length) {
			addOutput('', 'error');
			addOutput(`Found ${issues.length} issue(s):`, 'error');
			for (const i of issues) addOutput(`  • ${i}`, 'error');
		} else {
			addOutput('', 'success');
			addOutput('No issues found. Plugin config is valid.', 'success');
		}
	}

	function savePluginToDisk() {
		const libCode = editor?.getValue() || generateLibRs();
		const cargoCode = cargoEditor?.getValue() || generateCargoToml();
		send('save_plugin_files', { lib_rs: libCode, cargo_toml: cargoCode, dir: projectDir });
	}

	function installDll() {
		if (!lastDllPath) {
			toast('No DLL built yet — compile first', 'warning');
			return;
		}
		send('import_plugin', { path: lastDllPath });
		toast('Installing plugin DLL...', 'info');
	}

	// ── Code generation ──

	function generateCargoToml(): string {
		let toml = `[package]\nname = "${sanitize(pluginName).toLowerCase().replace(/\s+/g, '-')}"\nversion = "${pluginVersion}"\nedition = "2021"\n\n[lib]\ncrate-type = ["cdylib"]\n\n[dependencies]\nserde = { version = "1", features = ["derive"] }\nserde_json = "1"\n`;
		for (const dep of extraDeps) {
			if (!dep.name) continue;
			if (dep.features) {
				toml += `${dep.name} = { version = "${dep.version || '*'}", features = [${dep.features.split(',').map(f => `"${f.trim()}"`).join(', ')}] }\n`;
			} else {
				toml += `${dep.name} = "${dep.version || '*'}"\n`;
			}
		}
		return toml;
	}

	function sanitize(s: string): string {
		return s.replace(/"/g, '\\"');
	}

	function generateLibRs(): string {
		const blockCount = blocks.length;
		let code = `use std::collections::HashMap;\nuse std::ffi::{c_char, CStr, CString};\nuse std::sync::OnceLock;\n\n`;

		code += `// ── ABI structs (must match reqflow's plugin::abi) ──\n\n`;
		code += `#[repr(C)]\npub struct PluginInfo {\n    pub name: *const c_char,\n    pub version: *const c_char,\n    pub author: *const c_char,\n    pub description: *const c_char,\n    pub block_count: u32,\n}\n\n`;
		code += `unsafe impl Send for PluginInfo {}\nunsafe impl Sync for PluginInfo {}\n\n`;
		code += `#[repr(C)]\npub struct BlockInfo {\n    pub block_type_name: *const c_char,\n    pub label: *const c_char,\n    pub category: *const c_char,\n    pub color: *const c_char,\n    pub icon: *const c_char,\n    pub settings_schema_json: *const c_char,\n    pub default_settings_json: *const c_char,\n}\n\n`;
		code += `unsafe impl Send for BlockInfo {}\nunsafe impl Sync for BlockInfo {}\n\n`;
		code += `#[repr(C)]\npub struct ExecuteResult {\n    pub success: bool,\n    pub updated_variables_json: *const c_char,\n    pub log_message: *const c_char,\n    pub error_message: *const c_char,\n}\n\n`;

		code += `// ── Helpers ──\n\n`;
		code += `fn leak_cstring(s: &str) -> *const c_char {\n    CString::new(s).unwrap().into_raw() as *const c_char\n}\n\n`;
		code += `fn alloc_cstring(s: &str) -> *const c_char {\n    CString::new(s).unwrap().into_raw() as *const c_char\n}\n\n`;

		code += `// ── Plugin info ──\n\n`;
		code += `static PLUGIN_INFO: OnceLock<PluginInfo> = OnceLock::new();\n`;
		for (let i = 0; i < blockCount; i++) {
			code += `static BLOCK_INFO_${i}: OnceLock<BlockInfo> = OnceLock::new();\n`;
		}

		code += `\nfn get_plugin_info() -> &'static PluginInfo {\n`;
		code += `    PLUGIN_INFO.get_or_init(|| PluginInfo {\n`;
		code += `        name: leak_cstring("${sanitize(pluginName)}"),\n`;
		code += `        version: leak_cstring("${sanitize(pluginVersion)}"),\n`;
		code += `        author: leak_cstring("${sanitize(pluginAuthor)}"),\n`;
		code += `        description: leak_cstring("${sanitize(pluginDescription)}"),\n`;
		code += `        block_count: ${blockCount},\n`;
		code += `    })\n}\n\n`;

		for (let i = 0; i < blockCount; i++) {
			const b = blocks[i];
			const typeName = `${pluginName}.${b.name}`;
			const schema = generateSchemaJson(b);
			const defaults = generateDefaultsJson(b);

			code += `fn get_block_info_${i}() -> &'static BlockInfo {\n`;
			code += `    BLOCK_INFO_${i}.get_or_init(|| BlockInfo {\n`;
			code += `        block_type_name: leak_cstring("${sanitize(typeName)}"),\n`;
			code += `        label: leak_cstring("${sanitize(b.label)}"),\n`;
			code += `        category: leak_cstring("${sanitize(b.category)}"),\n`;
			code += `        color: leak_cstring("${sanitize(b.color)}"),\n`;
			code += `        icon: leak_cstring("puzzle"),\n`;
			code += `        settings_schema_json: leak_cstring(r#"${schema}"#),\n`;
			code += `        default_settings_json: leak_cstring(r#"${defaults}"#),\n`;
			code += `    })\n}\n\n`;
		}

		code += `// ── Exports ──\n\n`;
		code += `#[no_mangle]\npub extern "C" fn plugin_info() -> *const PluginInfo {\n    get_plugin_info() as *const PluginInfo\n}\n\n`;

		code += `#[no_mangle]\npub extern "C" fn plugin_block_info(index: u32) -> *const BlockInfo {\n`;
		code += `    match index {\n`;
		for (let i = 0; i < blockCount; i++) {
			code += `        ${i} => get_block_info_${i}() as *const BlockInfo,\n`;
		}
		code += `        _ => std::ptr::null(),\n`;
		code += `    }\n}\n\n`;

		code += `#[no_mangle]\npub extern "C" fn plugin_execute(\n    block_index: u32,\n    settings_json: *const c_char,\n    variables_json: *const c_char,\n) -> *const ExecuteResult {\n`;
		code += `    let settings_str = if settings_json.is_null() {\n        "{}".to_string()\n    } else {\n        unsafe { CStr::from_ptr(settings_json).to_string_lossy().to_string() }\n    };\n\n`;
		code += `    let vars_str = if variables_json.is_null() {\n        "{}".to_string()\n    } else {\n        unsafe { CStr::from_ptr(variables_json).to_string_lossy().to_string() }\n    };\n\n`;
		code += `    let settings: HashMap<String, serde_json::Value> =\n        serde_json::from_str(&settings_str).unwrap_or_default();\n`;
		code += `    let mut vars: HashMap<String, String> =\n        serde_json::from_str(&vars_str).unwrap_or_default();\n\n`;

		code += `    match block_index {\n`;
		for (let i = 0; i < blockCount; i++) {
			const b = blocks[i];
			code += `        ${i} => {\n`;
			code += `            // ── ${b.name}: ${b.label} ──\n`;
			const fieldNames: string[] = [];
			for (const f of b.settingsFields) {
				const vn = toSnake(f.name);
				fieldNames.push(vn);
				if (f.type === 'string') {
					code += `            let ${vn} = settings.get("${f.name}")\n                .and_then(|v| v.as_str())\n                .unwrap_or("${sanitize(f.default)}")\n                .to_string();\n`;
				} else if (f.type === 'number') {
					code += `            let ${vn} = settings.get("${f.name}")\n                .and_then(|v| v.as_f64())\n                .unwrap_or(${f.default || '0'}.0);\n`;
				} else if (f.type === 'boolean') {
					code += `            let ${vn} = settings.get("${f.name}")\n                .and_then(|v| v.as_bool())\n                .unwrap_or(${f.default || 'false'});\n`;
				}
			}
			code += `\n            // TODO: Implement your block logic here\n`;
			// Reference all extracted settings so there's no unused-variable warning
			if (fieldNames.length > 0) {
				code += `            let result_value = format!("${b.name}(${fieldNames.map(() => '{}').join(', ')})", ${fieldNames.join(', ')});\n\n`;
			} else {
				code += `            let result_value = "${b.name}".to_string();\n\n`;
			}
			code += `            vars.insert("PLUGIN_RESULT".to_string(), result_value.clone());\n`;
			code += `            let log_msg = format!("${b.name}: produced '{}'", result_value);\n`;
			code += `            make_result(true, &vars, &log_msg, "")\n`;
			code += `        }\n`;
		}
		code += `        _ => make_result(false, &vars, "", "Unknown block index"),\n`;
		code += `    }\n}\n\n`;

		code += `fn make_result(\n    success: bool,\n    vars: &HashMap<String, String>,\n    log: &str,\n    error: &str,\n) -> *const ExecuteResult {\n`;
		code += `    let vars_json = serde_json::to_string(vars).unwrap_or_else(|_| "{}".to_string());\n`;
		code += `    let result = Box::new(ExecuteResult {\n`;
		code += `        success,\n`;
		code += `        updated_variables_json: alloc_cstring(&vars_json),\n`;
		code += `        log_message: alloc_cstring(log),\n`;
		code += `        error_message: if error.is_empty() { std::ptr::null() } else { alloc_cstring(error) },\n`;
		code += `    });\n`;
		code += `    Box::into_raw(result) as *const ExecuteResult\n}\n\n`;

		code += `#[no_mangle]\npub extern "C" fn plugin_free_string(ptr: *const c_char) {\n    if !ptr.is_null() {\n        unsafe {\n            drop(CString::from_raw(ptr as *mut c_char));\n        }\n    }\n}\n`;

		return code;
	}

	function generateSchemaJson(b: PluginBlock): string {
		const props: Record<string, any> = {};
		for (const f of b.settingsFields) {
			props[f.name] = { type: f.type === 'number' ? 'number' : f.type === 'boolean' ? 'boolean' : 'string', title: f.name.replace(/_/g, ' '), default: f.default };
		}
		return JSON.stringify({ type: 'object', properties: props });
	}

	function generateDefaultsJson(b: PluginBlock): string {
		const defaults: Record<string, any> = {};
		for (const f of b.settingsFields) {
			if (f.type === 'number') defaults[f.name] = parseFloat(f.default) || 0;
			else if (f.type === 'boolean') defaults[f.name] = f.default === 'true';
			else defaults[f.name] = f.default;
		}
		return JSON.stringify(defaults);
	}

	function toSnake(s: string): string {
		return s.replace(/[A-Z]/g, (c, i) => (i ? '_' : '') + c.toLowerCase()).replace(/\s+/g, '_');
	}

	// ── Editor state ──

	let editorContainer: HTMLDivElement;
	let cargoContainer: HTMLDivElement;
	let editor = $state<Monaco.editor.IStandaloneCodeEditor | null>(null);
	let cargoEditor = $state<Monaco.editor.IStandaloneCodeEditor | null>(null);
	let monacoRef: typeof Monaco | null = null;

	let activeFile = $state<'lib.rs' | 'Cargo.toml'>('lib.rs');

	// Breakpoint decorations
	let breakpointDecorations = $state<string[]>([]);

	function toggleBreakpoint(lineNumber: number) {
		const next = new Set(breakpoints);
		if (next.has(lineNumber)) next.delete(lineNumber);
		else next.add(lineNumber);
		breakpoints = next;
		updateBreakpointDecorations();
	}

	function updateBreakpointDecorations() {
		if (!editor || !monacoRef) return;
		const newDecos = [...breakpoints].map(line => ({
			range: new monacoRef!.Range(line, 1, line, 1),
			options: {
				isWholeLine: true,
				linesDecorationsClassName: 'breakpoint-glyph',
				className: 'breakpoint-line-highlight',
			},
		}));
		breakpointDecorations = editor.deltaDecorations(breakpointDecorations, newDecos);
	}

	function regenerate() {
		if (editor) {
			const model = editor.getModel();
			if (model) model.setValue(generateLibRs());
		}
		if (cargoEditor) {
			const model = cargoEditor.getModel();
			if (model) model.setValue(generateCargoToml());
		}
	}

	function copyCode() {
		const val = activeFile === 'lib.rs' ? editor?.getValue() : cargoEditor?.getValue();
		if (val) {
			navigator.clipboard.writeText(val);
			toast(`${activeFile} copied to clipboard`, 'success');
		}
	}

	function downloadPlugin() {
		const libCode = editor?.getValue() || generateLibRs();
		const cargoCode = cargoEditor?.getValue() || generateCargoToml();
		downloadFile(`src/lib.rs`, libCode);
		setTimeout(() => downloadFile('Cargo.toml', cargoCode), 200);
		toast('Plugin files downloaded', 'success');
	}

	function downloadFile(name: string, content: string) {
		const blob = new Blob([content], { type: 'text/plain' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = name.replace(/\//g, '_');
		a.click();
		URL.revokeObjectURL(url);
	}

	function addBlock() {
		blocks = [...blocks, {
			name: `Block${blocks.length + 1}`,
			label: `Block ${blocks.length + 1}`,
			category: 'Utilities',
			color: '#9b59b6',
			settingsFields: [{ name: 'input_var', type: 'string', default: 'data.SOURCE' }],
		}];
	}

	function removeBlock(idx: number) {
		blocks = blocks.filter((_, i) => i !== idx);
	}

	function addField(blockIdx: number) {
		blocks[blockIdx].settingsFields = [...blocks[blockIdx].settingsFields, { name: 'new_field', type: 'string', default: '' }];
	}

	function removeField(blockIdx: number, fieldIdx: number) {
		blocks[blockIdx].settingsFields = blocks[blockIdx].settingsFields.filter((_, i) => i !== fieldIdx);
	}

	function addDep() {
		extraDeps = [...extraDeps, { name: '', version: '', features: '' }];
	}

	function removeDep(idx: number) {
		extraDeps = extraDeps.filter((_, i) => i !== idx);
	}

	function goBack() {
		app.showPluginBuilder = false;
	}

	onMount(async () => {
		const monaco = await import('monaco-editor');
		monacoRef = monaco;

		const EditorWorker = (await import('monaco-editor/esm/vs/editor/editor.worker?worker')).default;
		(self as any).MonacoEnvironment = {
			getWorker: () => new EditorWorker()
		};

		try {
			monaco.editor.defineTheme('reqflow-dark', {
				base: 'vs-dark',
				inherit: true,
				rules: [
					{ token: 'keyword', foreground: 'c586c0' },
					{ token: 'type', foreground: '4ec9b0' },
					{ token: 'string', foreground: 'ce9178' },
					{ token: 'number', foreground: 'b5cea8' },
					{ token: 'comment', foreground: '6a9955', fontStyle: 'italic' },
					{ token: 'variable', foreground: '9cdcfe' },
				],
				colors: {
					'editor.background': '#1a1a1d',
					'editor.foreground': '#cccccc',
					'editor.lineHighlightBackground': '#ffffff06',
					'editor.selectionBackground': '#264f78',
					'editorCursor.foreground': '#cccccc',
					'editorLineNumber.foreground': '#858585',
					'editorGutter.background': '#1a1a1d',
				},
			});
		} catch (_) { /* theme may already be defined */ }

		const editorOpts: Monaco.editor.IStandaloneEditorConstructionOptions = {
			theme: 'reqflow-dark',
			minimap: { enabled: false },
			fontSize: 12,
			fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', Consolas, monospace",
			lineNumbers: 'on',
			scrollBeyondLastLine: false,
			automaticLayout: true,
			tabSize: 4,
			padding: { top: 8, bottom: 8 },
			overviewRulerLanes: 0,
			hideCursorInOverviewRuler: true,
			scrollbar: { verticalScrollbarSize: 6, horizontalScrollbarSize: 6 },
			folding: true,
			wordWrap: 'off',
			matchBrackets: 'always',
			bracketPairColorization: { enabled: true },
			glyphMargin: true,
		};

		editor = monaco.editor.create(editorContainer, {
			...editorOpts,
			value: generateLibRs(),
			language: 'rust',
		});

		// Click glyph margin to toggle breakpoints
		editor.onMouseDown((e) => {
			if (e.target.type === monaco.editor.MouseTargetType.GUTTER_GLYPH_MARGIN) {
				const line = e.target.position?.lineNumber;
				if (line) toggleBreakpoint(line);
			}
		});

		cargoEditor = monaco.editor.create(cargoContainer, {
			...editorOpts,
			value: generateCargoToml(),
			language: 'toml',
		});
	});

	onDestroy(() => {
		if (editor) { editor.dispose(); editor = null; }
		if (cargoEditor) { cargoEditor.dispose(); cargoEditor = null; }
		monacoRef = null;
		app._compileOutputCallback = null;
	});

	// Documentation content
	const DOCS: Record<Section, string> = {
		config: '',
		'getting-started': `
<h3 class="text-sm font-semibold text-foreground mb-3">Getting Started with Plugins</h3>
<p class="text-[12px] text-muted-foreground mb-3">reqflow plugins are <strong>Rust DLLs</strong> (.dll on Windows) that export a C-compatible ABI. The host loads your DLL at runtime, queries it for block definitions, and calls your execute function during pipeline runs.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">How It Works</h3>
<ol class="text-[12px] text-foreground/85 space-y-2 pl-5 list-decimal mb-4">
<li><strong>reqflow scans</strong> the plugins directory for <code>.dll</code> files</li>
<li>For each DLL, it calls <code>plugin_info()</code> to get your plugin's name, version, and block count</li>
<li>It calls <code>plugin_block_info(index)</code> for each block to get labels, categories, settings schema, and defaults</li>
<li>Your blocks appear in the Block Palette alongside built-in blocks</li>
<li>During execution, reqflow calls <code>plugin_execute(block_index, settings, variables)</code> with the current pipeline state</li>
<li>Your function processes data and returns updated variables</li>
</ol>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Project Structure</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">my-plugin/
  Cargo.toml          # crate-type = ["cdylib"]
  src/
    lib.rs            # All 4 required exports</pre>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Required Exports</h3>
<table class="w-full text-[11px] border border-border rounded overflow-hidden mb-3">
<thead><tr class="bg-accent/10 border-b border-border">
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Function</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Purpose</th>
</tr></thead>
<tbody>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono text-foreground">plugin_info()</td><td class="px-3 py-1.5 text-foreground/80">Returns plugin metadata (name, version, block count)</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono text-foreground">plugin_block_info(index)</td><td class="px-3 py-1.5 text-foreground/80">Returns block definition for given index</td></tr>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono text-foreground">plugin_execute(index, settings, vars)</td><td class="px-3 py-1.5 text-foreground/80">Runs block logic, returns updated variables</td></tr>
<tr class="bg-accent/5"><td class="px-3 py-1.5 font-mono text-foreground">plugin_free_string(ptr)</td><td class="px-3 py-1.5 text-foreground/80">Frees a CString allocated by the plugin</td></tr>
</tbody>
</table>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Quick Workflow</h3>
<ol class="text-[12px] text-foreground/85 space-y-1 pl-5 list-decimal">
<li>Configure your plugin name, blocks, and settings in the <strong>Plugin Config</strong> tab</li>
<li>Click <strong>Regenerate</strong> to update the generated code</li>
<li>Copy or download the files</li>
<li>Add your custom logic in the <code>// TODO</code> sections</li>
<li>Build: <kbd>cargo build --release</kbd></li>
<li>Copy the DLL to your plugins directory</li>
<li>Restart reqflow or re-scan plugins</li>
</ol>`,

		'abi-reference': `
<h3 class="text-sm font-semibold text-foreground mb-3">ABI Reference</h3>
<p class="text-[12px] text-muted-foreground mb-3">All communication between reqflow and plugins uses C-compatible <code>#[repr(C)]</code> structs with raw <code>*const c_char</code> pointers. This ensures ABI stability across different Rust compiler versions.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">PluginInfo</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">#[repr(C)]
pub struct PluginInfo {
    pub name: *const c_char,
    pub version: *const c_char,
    pub author: *const c_char,
    pub description: *const c_char,
    pub block_count: u32,
}</pre>
<p class="text-[12px] text-muted-foreground mb-3"><strong>Lifetime:</strong> The returned pointer must be valid for the entire DLL lifetime. Use <code>OnceLock</code> + <code>leak_cstring()</code> for static allocation.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">BlockInfo</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">#[repr(C)]
pub struct BlockInfo {
    pub block_type_name: *const c_char,
    pub label: *const c_char,
    pub category: *const c_char,
    pub color: *const c_char,
    pub icon: *const c_char,
    pub settings_schema_json: *const c_char,
    pub default_settings_json: *const c_char,
}</pre>
<p class="text-[12px] text-muted-foreground mb-3"><strong>block_type_name</strong> must be <code>"PluginName.BlockName"</code> format. reqflow uses this to route execution calls to the correct plugin and block.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">ExecuteResult</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">#[repr(C)]
pub struct ExecuteResult {
    pub success: bool,
    pub updated_variables_json: *const c_char,
    pub log_message: *const c_char,
    pub error_message: *const c_char,
}</pre>
<p class="text-[12px] text-muted-foreground mb-3"><strong>Memory:</strong> These strings are allocated by the plugin and freed by reqflow via <code>plugin_free_string()</code>. Use <code>CString::new(s).unwrap().into_raw()</code> to allocate.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Memory Safety Rules</h3>
<ul class="text-[12px] text-foreground/85 space-y-1 pl-5 list-disc">
<li><strong>Static data</strong> (PluginInfo, BlockInfo): Use <code>OnceLock</code> + <code>leak_cstring()</code> — these are never freed</li>
<li><strong>Result data</strong> (ExecuteResult): Use <code>CString::new().into_raw()</code> — freed by host via <code>plugin_free_string</code></li>
<li><strong>Box&lt;ExecuteResult&gt;</strong>: Allocate with <code>Box::into_raw()</code> — host reads then drops</li>
<li>Never return stack pointers or references to temporaries</li>
</ul>`,

		'block-definition': `
<h3 class="text-sm font-semibold text-foreground mb-3">Block Definition</h3>
<p class="text-[12px] text-muted-foreground mb-3">Each block your plugin provides is described by a <code>BlockInfo</code> struct returned from <code>plugin_block_info(index)</code>. The index is 0-based and must be less than the <code>block_count</code> you reported in <code>plugin_info()</code>.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Type Name Convention</h3>
<p class="text-[12px] text-muted-foreground mb-3">The <code>block_type_name</code> must follow the <code>PluginName.BlockName</code> convention. This is how reqflow identifies and routes execution to your block. Example: <code>"MyPlugin.ReverseString"</code>.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Categories</h3>
<p class="text-[12px] text-muted-foreground mb-2">Choose a category that matches your block's purpose:</p>
<div class="flex flex-wrap gap-1.5 mb-3">
${CATEGORIES.map(c => `<span class="text-[10px] px-2 py-0.5 rounded border border-border text-foreground/70">${c}</span>`).join('\n')}
</div>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Icon Names</h3>
<p class="text-[12px] text-muted-foreground mb-3">Icons use <a href="https://lucide.dev/icons/" class="text-blue-400 underline">Lucide icon names</a> (lowercase, hyphenated). Examples: <code>puzzle</code>, <code>repeat</code>, <code>globe</code>, <code>database</code>, <code>key</code>, <code>shield</code>.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Example: Multiple Blocks</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">// Plugin with 2 blocks
fn get_plugin_info() -> &'static PluginInfo {
    PLUGIN_INFO.get_or_init(|| PluginInfo {
        name: leak_cstring("StringTools"),
        block_count: 2,
        // ...
    })
}

fn plugin_block_info(index: u32) -> *const BlockInfo {
    match index {
        0 => get_reverse_block(),
        1 => get_uppercase_block(),
        _ => std::ptr::null(),
    }
}</pre>`,

		'execution': `
<h3 class="text-sm font-semibold text-foreground mb-3">Execution Flow</h3>
<p class="text-[12px] text-muted-foreground mb-3">When reqflow executes a pipeline and encounters one of your plugin blocks, it calls <code>plugin_execute()</code> with the block index, current settings, and the full variable map.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Input Parameters</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">pub extern "C" fn plugin_execute(
    block_index: u32,
    settings_json: *const c_char,
    variables_json: *const c_char,
) -> *const ExecuteResult</pre>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Variables Map</h3>
<p class="text-[12px] text-muted-foreground mb-2">The variables JSON is a flat <code>HashMap&lt;String, String&gt;</code> containing all pipeline state:</p>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">{
    "data.USER": "john@example.com",
    "data.PASS": "secret123",
    "data.SOURCE": "&lt;html&gt;...&lt;/html&gt;",
    "data.RESPONSECODE": "200",
    "data.SOURCE.HEADERS": "{...}",
    "data.SOURCE.COOKIES": "{...}",
    "CAPTURE_email": "john@example.com"
}</pre>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Common Variable Names</h3>
<table class="w-full text-[11px] border border-border rounded overflow-hidden mb-3">
<thead><tr class="bg-accent/10 border-b border-border">
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Variable</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Contains</th>
</tr></thead>
<tbody>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">data.SOURCE</td><td class="px-3 py-1.5 text-foreground/80">Last HTTP response body</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono">data.RESPONSECODE</td><td class="px-3 py-1.5 text-foreground/80">HTTP status code</td></tr>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">data.USER / data.PASS</td><td class="px-3 py-1.5 text-foreground/80">Current data line fields</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono">data.SOURCE.HEADERS</td><td class="px-3 py-1.5 text-foreground/80">Response headers (JSON)</td></tr>
<tr><td class="px-3 py-1.5 font-mono">data.SOURCE.COOKIES</td><td class="px-3 py-1.5 text-foreground/80">Response cookies (JSON)</td></tr>
</tbody>
</table>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Return Pattern</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">// Return your modified variables
let mut vars = parse_variables(variables_json);

// Read input
let source = vars.get("data.SOURCE").cloned().unwrap_or_default();

// Do your processing
let result = my_transform(&source);

// Write output
vars.insert("PLUGIN_RESULT".to_string(), result);
vars.insert("CAPTURE_myfield".to_string(), "captured_value".to_string());

// Return success
make_result(true, &vars, "Processed OK", "")</pre>
<p class="text-[12px] text-muted-foreground mb-2"><strong>Captures:</strong> Any variable prefixed with <code>CAPTURE_</code> will be saved to the output file when the pipeline finishes with a "hit" status.</p>`,

		'settings-schema': `
<h3 class="text-sm font-semibold text-foreground mb-3">Settings Schema</h3>
<p class="text-[12px] text-muted-foreground mb-3">The <code>settings_schema_json</code> field in <code>BlockInfo</code> defines what the settings panel shows when the user clicks on your block. It uses <strong>JSON Schema</strong> (draft-07 compatible).</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Supported Field Types</h3>
<table class="w-full text-[11px] border border-border rounded overflow-hidden mb-3">
<thead><tr class="bg-accent/10 border-b border-border">
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">JSON Schema Type</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">UI Control</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Example</th>
</tr></thead>
<tbody>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">string</td><td class="px-3 py-1.5 text-foreground/80">Text input</td><td class="px-3 py-1.5 font-mono text-[10px]">{"type":"string","title":"URL"}</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono">number</td><td class="px-3 py-1.5 text-foreground/80">Number input</td><td class="px-3 py-1.5 font-mono text-[10px]">{"type":"number","title":"Timeout"}</td></tr>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">boolean</td><td class="px-3 py-1.5 text-foreground/80">Toggle switch</td><td class="px-3 py-1.5 font-mono text-[10px]">{"type":"boolean","title":"Verbose"}</td></tr>
<tr class="bg-accent/5"><td class="px-3 py-1.5 font-mono">string + enum</td><td class="px-3 py-1.5 text-foreground/80">Dropdown select</td><td class="px-3 py-1.5 font-mono text-[10px]">{"type":"string","enum":["a","b"]}</td></tr>
</tbody>
</table>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Full Example</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">{
    "type": "object",
    "properties": {
        "input_var": {
            "type": "string",
            "title": "Input Variable",
            "default": "data.SOURCE"
        },
        "timeout_ms": {
            "type": "number",
            "title": "Timeout (ms)",
            "default": 5000
        },
        "mode": {
            "type": "string",
            "title": "Mode",
            "enum": ["fast", "thorough", "balanced"],
            "default": "fast"
        },
        "verbose": {
            "type": "boolean",
            "title": "Verbose Logging",
            "default": false
        }
    }
}</pre>
<p class="text-[12px] text-muted-foreground mb-3">The <code>default_settings_json</code> should match the schema defaults:</p>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">{"input_var":"data.SOURCE","timeout_ms":5000,"mode":"fast","verbose":false}</pre>`,

		'dependencies': `
<h3 class="text-sm font-semibold text-foreground mb-3">Dependencies</h3>
<p class="text-[12px] text-muted-foreground mb-3">Your plugin is a standard Rust crate. You can add any crate from <a href="https://crates.io" class="text-blue-400 underline">crates.io</a> to your <code>Cargo.toml</code>.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Required Dependencies</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"</pre>
<p class="text-[12px] text-muted-foreground mb-3"><code>serde</code> and <code>serde_json</code> are mandatory — they handle the JSON encoding/decoding for settings and variables.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Common Useful Crates</h3>
<table class="w-full text-[11px] border border-border rounded overflow-hidden mb-3">
<thead><tr class="bg-accent/10 border-b border-border">
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Crate</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Purpose</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Cargo.toml</th>
</tr></thead>
<tbody>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">regex</td><td class="px-3 py-1.5 text-foreground/80">Regular expressions</td><td class="px-3 py-1.5 font-mono text-[10px]">regex = "1"</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono">scraper</td><td class="px-3 py-1.5 text-foreground/80">HTML/CSS parsing</td><td class="px-3 py-1.5 font-mono text-[10px]">scraper = "0.20"</td></tr>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">sha2 / md5</td><td class="px-3 py-1.5 text-foreground/80">Hashing</td><td class="px-3 py-1.5 font-mono text-[10px]">sha2 = "0.10"</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono">base64</td><td class="px-3 py-1.5 text-foreground/80">Base64 encode/decode</td><td class="px-3 py-1.5 font-mono text-[10px]">base64 = "0.22"</td></tr>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">rand</td><td class="px-3 py-1.5 text-foreground/80">Random generation</td><td class="px-3 py-1.5 font-mono text-[10px]">rand = "0.8"</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono">url</td><td class="px-3 py-1.5 text-foreground/80">URL parsing</td><td class="px-3 py-1.5 font-mono text-[10px]">url = "2"</td></tr>
<tr><td class="px-3 py-1.5 font-mono">chrono</td><td class="px-3 py-1.5 text-foreground/80">Date/time</td><td class="px-3 py-1.5 font-mono text-[10px]">chrono = "0.4"</td></tr>
</tbody>
</table>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Important: crate-type</h3>
<p class="text-[12px] text-muted-foreground mb-3">Your <code>Cargo.toml</code> <strong>must</strong> include <code>crate-type = ["cdylib"]</code> under <code>[lib]</code>. This tells Cargo to produce a C-compatible dynamic library (.dll) instead of a Rust-only .rlib.</p>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">[lib]
crate-type = ["cdylib"]</pre>
<p class="text-[12px] text-muted-foreground mb-3"><strong>Note:</strong> Async crates (tokio, reqwest) work but your <code>plugin_execute</code> is called synchronously. If you need async, create a tokio runtime inside your execute function:</p>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">let rt = tokio::runtime::Runtime::new().unwrap();
let result = rt.block_on(async {
    // async code here
});</pre>`,

		'building': `
<h3 class="text-sm font-semibold text-foreground mb-3">Build & Load</h3>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">1. Build Your Plugin</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">cd my-plugin
cargo build --release</pre>
<p class="text-[12px] text-muted-foreground mb-3">The compiled DLL will be at:<br><code>target/release/my_plugin.dll</code></p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">2. Copy to Plugins Directory</h3>
<p class="text-[12px] text-muted-foreground mb-3">Copy the DLL to your configured plugins directory. You can set this path in <strong>Settings &gt; Plugins Path</strong>.</p>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3"># Default location (next to reqflow.exe)
cp target/release/my_plugin.dll ./plugins/</pre>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">3. Load in reqflow</h3>
<p class="text-[12px] text-muted-foreground mb-3">Plugins are loaded at startup. To reload after changes:</p>
<ul class="text-[12px] text-foreground/85 space-y-1 pl-5 list-disc mb-3">
<li>Restart reqflow, or</li>
<li>Use <strong>File &gt; Import Plugin (.dll)</strong> to load a specific DLL</li>
</ul>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">4. Verify</h3>
<p class="text-[12px] text-muted-foreground mb-3">After loading, your blocks should appear in the Block Palette under their configured category. Add one to a pipeline and check the Debug panel to verify execution.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Troubleshooting</h3>
<table class="w-full text-[11px] border border-border rounded overflow-hidden mb-3">
<thead><tr class="bg-accent/10 border-b border-border">
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Issue</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Cause & Fix</th>
</tr></thead>
<tbody>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 text-foreground">DLL not found</td><td class="px-3 py-1.5 text-foreground/80">Check plugins directory path in Settings.</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 text-foreground">Missing export error</td><td class="px-3 py-1.5 text-foreground/80">Ensure all 4 functions are <code>#[no_mangle] pub extern "C"</code> and crate-type is <code>cdylib</code>.</td></tr>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 text-foreground">Crash on load</td><td class="px-3 py-1.5 text-foreground/80">Check for null pointer returns.</td></tr>
<tr class="bg-accent/5"><td class="px-3 py-1.5 text-foreground">Settings not showing</td><td class="px-3 py-1.5 text-foreground/80">Validate your JSON Schema string.</td></tr>
</tbody>
</table>`,
	};
</script>

<div class="h-full flex flex-col overflow-hidden bg-surface">
	<!-- Top bar -->
	<div class="flex items-center gap-2 px-3 py-1.5 border-b border-border bg-surface shrink-0">
		<button
			class="skeu-btn flex items-center gap-1.5 text-[11px] text-muted-foreground"
			onclick={goBack}
		>
			<ArrowLeft size={12} />
			Back
		</button>
		<div class="w-px h-3.5 bg-border mx-0.5"></div>
		<Puzzle size={14} class="text-muted-foreground" />
		<span class="text-[12px] font-semibold text-foreground">Plugin Builder</span>
		<div class="w-px h-3.5 bg-border mx-0.5"></div>

		<!-- IDE toolbar -->
		<button
			class="skeu-btn flex items-center gap-1 text-[10px] text-muted-foreground"
			onclick={regenerate}
			title="Regenerate code from config"
		>
			<RefreshCw size={10} />Regen
		</button>
		<button
			class="skeu-btn flex items-center gap-1 text-[10px] {isCompiling ? 'opacity-50' : 'text-green'}"
			onclick={compilePlugin}
			disabled={isCompiling}
			title="Compile plugin (cargo build)"
		>
			<Hammer size={10} />{isCompiling ? 'Building...' : 'Build'}
		</button>
		<button
			class="skeu-btn flex items-center gap-1 text-[10px] text-blue-400"
			onclick={debugInspect}
			title="Inspect plugin config & validate"
		>
			<Bug size={10} />Inspect
		</button>
		{#if lastBuildSuccess && lastDllPath}
			<button
				class="skeu-btn flex items-center gap-1 text-[10px] text-orange-400"
				onclick={installDll}
				title="Install built DLL into plugins"
			>
				<Play size={10} />Install
			</button>
		{/if}

		<div class="flex-1"></div>

		<button
			class="skeu-btn flex items-center gap-1 text-[10px] text-muted-foreground"
			onclick={savePluginToDisk}
			title="Save plugin project to disk"
		>
			<FolderOpen size={10} />Save Project
		</button>
		<button class="skeu-btn flex items-center gap-1 text-[10px] text-muted-foreground" onclick={copyCode}>
			<Copy size={10} />Copy
		</button>
		<button class="skeu-btn flex items-center gap-1 text-[10px] text-muted-foreground" onclick={downloadPlugin}>
			<Download size={10} />Download
		</button>
	</div>

	<!-- Main content -->
	<div class="flex flex-1 overflow-hidden">
		<!-- Left sidebar -->
		<div class="w-48 border-r border-border overflow-y-auto shrink-0 bg-surface/50 flex flex-col">
			<div class="py-1">
				{#each SECTIONS as section}
					{@const Icon = section.icon}
					<button
						class="w-full text-left px-3 py-1.5 text-[11px] hover:bg-accent/30 transition-colors flex items-center gap-2 {activeSection === section.id ? 'bg-accent/40 text-foreground font-medium' : 'text-muted-foreground'}"
						onclick={() => { activeSection = section.id; }}
					>
						<Icon size={12} class="shrink-0" />
						<span class="truncate">{section.label}</span>
					</button>
				{/each}
			</div>

			{#if app.pluginMetas.length > 0}
				<div class="mt-auto border-t border-border p-2">
					<div class="text-[9px] uppercase tracking-wider text-muted-foreground font-semibold mb-1">Loaded Plugins</div>
					{#each app.pluginMetas as pm}
						<div class="text-[10px] text-foreground/70 truncate" title={pm.dll_path}>{pm.name} v{pm.version}</div>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Center: config form or docs (resizable) -->
		{#if activeSection === 'config'}
			<div class="border-r border-border overflow-y-auto shrink-0 p-3 space-y-3" style="width: {centerPanelWidth}px">
				<!-- Plugin metadata -->
				<details open class="group">
					<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
						<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
						Plugin Metadata
					</summary>
					<div class="mt-2 space-y-2">
						<label class="block">
							<span class="text-[10px] text-muted-foreground">Plugin Name</span>
							<input class="skeu-input w-full text-[11px] mt-0.5 truncate" bind:value={pluginName} placeholder="MyPlugin" />
						</label>
						<label class="block">
							<span class="text-[10px] text-muted-foreground">Version</span>
							<input class="skeu-input w-full text-[11px] mt-0.5 truncate" bind:value={pluginVersion} placeholder="0.1.0" />
						</label>
						<label class="block">
							<span class="text-[10px] text-muted-foreground">Author</span>
							<input class="skeu-input w-full text-[11px] mt-0.5 truncate" bind:value={pluginAuthor} placeholder="Your name" />
						</label>
						<label class="block">
							<span class="text-[10px] text-muted-foreground">Description</span>
							<input class="skeu-input w-full text-[11px] mt-0.5 truncate" bind:value={pluginDescription} placeholder="What does this plugin do?" />
						</label>
					</div>
				</details>

				<!-- Blocks -->
				<details open class="group">
					<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
						<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
						Blocks ({blocks.length})
					</summary>
					<div class="mt-2 space-y-3">
						{#each blocks as block, bi}
							<div class="border border-border rounded p-2 space-y-1.5 bg-accent/5 overflow-hidden">
								<div class="flex items-center gap-1.5">
									<div class="w-2.5 h-2.5 rounded shrink-0" style="background: {block.color}"></div>
									<span class="text-[11px] font-medium text-foreground flex-1 truncate">{block.name}</span>
									{#if blocks.length > 1}
										<button class="p-0.5 rounded hover:bg-accent/30 text-muted-foreground shrink-0" onclick={() => removeBlock(bi)} title="Remove block">
											<Trash2 size={10} />
										</button>
									{/if}
								</div>
								<div class="grid grid-cols-2 gap-1.5">
									<label class="block min-w-0">
										<span class="text-[9px] text-muted-foreground">Name</span>
										<input class="skeu-input w-full text-[10px] mt-0.5 truncate" bind:value={block.name} placeholder="BlockName" />
									</label>
									<label class="block min-w-0">
										<span class="text-[9px] text-muted-foreground">Label</span>
										<input class="skeu-input w-full text-[10px] mt-0.5 truncate" bind:value={block.label} placeholder="Display Name" />
									</label>
								</div>
								<div class="grid grid-cols-2 gap-1.5">
									<div class="min-w-0">
										<span class="text-[9px] text-muted-foreground block mb-0.5">Category</span>
										<SkeuSelect
											value={block.category}
											onValueChange={(v) => { block.category = v; }}
											options={CATEGORIES.map(c => ({ value: c, label: c }))}
											class="w-full text-[10px]"
										/>
									</div>
									<label class="block min-w-0">
										<span class="text-[9px] text-muted-foreground">Color</span>
										<div class="flex items-center gap-1 mt-0.5">
											<input type="color" class="w-5 h-5 rounded border border-border cursor-pointer shrink-0" bind:value={block.color} />
											<input class="skeu-input flex-1 min-w-0 text-[10px]" bind:value={block.color} />
										</div>
									</label>
								</div>

								<!-- Settings fields -->
								<div class="mt-1">
									<div class="flex items-center gap-1 mb-1">
										<span class="text-[9px] text-muted-foreground uppercase tracking-wider">Settings Fields</span>
										<button class="p-0.5 rounded hover:bg-accent/30 text-muted-foreground ml-auto shrink-0" onclick={() => addField(bi)} title="Add field">
											<Plus size={10} />
										</button>
									</div>
									{#each block.settingsFields as field, fi}
										<div class="flex items-center gap-1 mb-1">
											<input class="skeu-input flex-1 min-w-0 text-[10px]" bind:value={field.name} placeholder="field_name" />
											<div class="w-14 shrink-0">
												<SkeuSelect
													value={field.type}
													onValueChange={(v) => { field.type = v; }}
													options={FIELD_TYPES}
													class="w-full text-[10px]"
												/>
											</div>
											<input class="skeu-input w-16 min-w-0 text-[10px]" bind:value={field.default} placeholder="default" />
											<button class="p-0.5 rounded hover:bg-accent/30 text-muted-foreground shrink-0" onclick={() => removeField(bi, fi)}>
												<Trash2 size={9} />
											</button>
										</div>
									{/each}
								</div>
							</div>
						{/each}
						<button class="skeu-btn w-full text-[10px] text-muted-foreground flex items-center justify-center gap-1 py-1.5" onclick={addBlock}>
							<Plus size={10} />Add Block
						</button>
					</div>
				</details>

				<!-- Extra dependencies -->
				<details class="group">
					<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
						<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
						Extra Dependencies ({extraDeps.length})
					</summary>
					<div class="mt-2 space-y-1.5">
						{#each extraDeps as dep, di}
							<div class="flex items-center gap-1">
								<input class="skeu-input flex-1 min-w-0 text-[10px]" bind:value={dep.name} placeholder="crate_name" />
								<input class="skeu-input w-14 min-w-0 text-[10px]" bind:value={dep.version} placeholder="ver" />
								<input class="skeu-input w-20 min-w-0 text-[10px]" bind:value={dep.features} placeholder="features" />
								<button class="p-0.5 rounded hover:bg-accent/30 text-muted-foreground shrink-0" onclick={() => removeDep(di)}>
									<Trash2 size={9} />
								</button>
							</div>
						{/each}
						<button class="skeu-btn w-full text-[10px] text-muted-foreground flex items-center justify-center gap-1 py-1" onclick={addDep}>
							<Plus size={10} />Add Dependency
						</button>
					</div>
				</details>

				<!-- Build settings -->
				<details class="group">
					<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
						<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
						Build Settings
					</summary>
					<div class="mt-2 space-y-2">
						<label class="block min-w-0">
							<span class="text-[10px] text-muted-foreground">Project Directory (optional)</span>
							<input class="skeu-input w-full text-[10px] mt-0.5 truncate" bind:value={projectDir} placeholder="temp dir if empty" />
						</label>
						<label class="flex items-center gap-2 text-[10px] text-foreground/80">
							<input type="checkbox" bind:checked={buildRelease} class="accent-green" />
							Release mode (--release)
						</label>
					</div>
				</details>
			</div>
		{:else}
			<!-- Documentation content -->
			<div class="border-r border-border overflow-y-auto shrink-0 p-4 guide-content" style="width: {centerPanelWidth}px">
				{@html DOCS[activeSection]}
			</div>
		{/if}

		<!-- Vertical resize handle between center and editor -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="groove-v resize-handle w-[3px] shrink-0" onmousedown={startResizeCenter}></div>

		<!-- Right: Code editor + output -->
		<div class="flex-1 flex flex-col min-w-0">
			<!-- File tabs + breakpoint indicator -->
			<div class="flex items-center gap-0 px-1 py-0.5 bg-surface border-b border-border shrink-0">
				<button
					class="px-3 py-1 text-[11px] rounded-t transition-colors {activeFile === 'lib.rs' ? 'bg-[#1a1a1d] text-foreground font-medium border border-border border-b-transparent -mb-px' : 'text-muted-foreground hover:text-foreground'}"
					onclick={() => { activeFile = 'lib.rs'; }}
				>
					src/lib.rs
				</button>
				<button
					class="px-3 py-1 text-[11px] rounded-t transition-colors {activeFile === 'Cargo.toml' ? 'bg-[#1a1a1d] text-foreground font-medium border border-border border-b-transparent -mb-px' : 'text-muted-foreground hover:text-foreground'}"
					onclick={() => { activeFile = 'Cargo.toml'; }}
				>
					Cargo.toml
				</button>
				<div class="flex-1"></div>
				{#if breakpoints.size > 0}
					<div class="flex items-center gap-1 text-[9px] text-orange-400 mr-2">
						<CircleDot size={9} />
						{breakpoints.size} breakpoint{breakpoints.size > 1 ? 's' : ''}
					</div>
				{/if}
				<button
					class="p-1 rounded hover:bg-accent/30 text-muted-foreground"
					onclick={() => { showOutput = !showOutput; }}
					title={showOutput ? 'Hide output' : 'Show output'}
				>
					<Terminal size={12} />
				</button>
			</div>

			<!-- Editors -->
			<div class="flex-1 overflow-hidden" style="display: {activeFile === 'lib.rs' ? 'block' : 'none'}" bind:this={editorContainer}></div>
			<div class="flex-1 overflow-hidden" style="display: {activeFile === 'Cargo.toml' ? 'block' : 'none'}" bind:this={cargoContainer}></div>

			<!-- Output panel (terminal) -->
			{#if showOutput}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="groove-h resize-handle-h h-[3px] shrink-0" onmousedown={startResizeOutput}></div>
				<div class="bg-[#1a1a1d] shrink-0" style="height: {outputHeight}px;">
					<div class="flex items-center gap-1.5 px-2 py-1 border-b border-border/50 bg-surface/30">
						<Terminal size={10} class="text-muted-foreground" />
						<span class="text-[10px] text-muted-foreground font-medium">Output</span>
						{#if isCompiling}
							<span class="text-[9px] text-yellow-400 animate-pulse">compiling...</span>
						{:else if lastBuildSuccess === true}
							<span class="text-[9px] text-green">build ok</span>
						{:else if lastBuildSuccess === false}
							<span class="text-[9px] text-red-400">build failed</span>
						{/if}
						<div class="flex-1"></div>
						<button class="text-[9px] text-muted-foreground hover:text-foreground" onclick={clearOutput}>Clear</button>
						<button class="p-0.5 rounded hover:bg-accent/30 text-muted-foreground" onclick={() => { showOutput = false; }}>
							<X size={10} />
						</button>
					</div>
					<div class="output-scroll overflow-auto h-[calc(100%-26px)] px-2 py-1 font-mono text-[11px]" bind:this={outputEl}>
						{#each outputLines as line}
							<div class="leading-[18px] {line.type === 'error' ? 'text-red-400' : line.type === 'success' ? 'text-green' : line.type === 'cmd' ? 'text-blue-400' : 'text-foreground/80'}">
								{line.text || '\u00A0'}
							</div>
						{/each}
						{#if outputLines.length === 0}
							<div class="text-muted-foreground/50 text-[10px] mt-2">Click Build to compile your plugin, or Inspect to validate config.</div>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.guide-content :global(h3) {
		color: var(--foreground);
	}
	.guide-content :global(p) {
		color: var(--muted-foreground);
		overflow-wrap: break-word;
		word-break: break-word;
	}
	.guide-content :global(li) {
		color: var(--foreground);
		opacity: 0.85;
		overflow-wrap: break-word;
		word-break: break-word;
	}
	.guide-content :global(code) {
		background: var(--accent);
		padding: 1px 4px;
		border-radius: 3px;
		font-size: 10px;
		font-family: var(--font-mono, monospace);
		word-break: break-all;
	}
	.guide-content :global(pre) {
		color: var(--foreground);
		opacity: 0.9;
		overflow-x: auto;
		white-space: pre;
		max-width: 100%;
		user-select: text;
	}
	.guide-content :global(pre code) {
		background: none;
		padding: 0;
	}
	.guide-content :global(table) {
		border: 1px solid var(--border);
		border-radius: 6px;
		overflow: hidden;
		table-layout: fixed;
		width: 100%;
		word-break: break-word;
	}
	.guide-content :global(td),
	.guide-content :global(th) {
		color: var(--foreground);
		overflow-wrap: break-word;
	}
	.guide-content :global(kbd) {
		background: var(--accent);
		padding: 1px 6px;
		border-radius: 3px;
		border: 1px solid var(--border);
		font-size: 10px;
		font-family: var(--font-mono, monospace);
	}
	.guide-content :global(ol) {
		list-style-type: decimal;
	}
	.guide-content :global(ul) {
		list-style-type: disc;
	}

	/* Output panel — text must be selectable/copyable */
	.output-scroll {
		user-select: text;
		cursor: text;
	}
	.output-scroll :global(div) {
		user-select: text;
	}

	/* Guide content — make docs text selectable */
	.guide-content {
		user-select: text;
	}

	/* Breakpoint glyph in editor gutter */
	:global(.breakpoint-glyph) {
		background: #e53935;
		border-radius: 50%;
		width: 8px !important;
		height: 8px !important;
		margin-left: 4px;
		margin-top: 4px;
	}
	:global(.breakpoint-line-highlight) {
		background: rgba(229, 57, 53, 0.08) !important;
	}
</style>
