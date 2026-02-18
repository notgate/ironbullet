<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import { toast } from '$lib/toast.svelte';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import Copy from '@lucide/svelte/icons/copy';
	import Download from '@lucide/svelte/icons/download';
	import Puzzle from '@lucide/svelte/icons/puzzle';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Box from '@lucide/svelte/icons/box';
	import Zap from '@lucide/svelte/icons/zap';
	import Settings from '@lucide/svelte/icons/settings';
	import Package from '@lucide/svelte/icons/package';
	import Wrench from '@lucide/svelte/icons/wrench';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import Play from '@lucide/svelte/icons/play';
	import Hammer from '@lucide/svelte/icons/hammer';
	import Bug from '@lucide/svelte/icons/bug';
	import FolderOpen from '@lucide/svelte/icons/folder-open';
	import CircleDot from '@lucide/svelte/icons/circle-dot';
	import Terminal from '@lucide/svelte/icons/terminal';
	import type * as Monaco from 'monaco-editor';

	import { type PluginBlock, type Section, type SectionIcon } from '$lib/components/plugin-builder/types';
	import { generateCargoToml, generateLibRs } from '$lib/components/plugin-builder/codegen';
	import ConfigSection from '$lib/components/plugin-builder/ConfigSection.svelte';
	import GettingStartedSection from '$lib/components/plugin-builder/GettingStartedSection.svelte';
	import AbiReferenceSection from '$lib/components/plugin-builder/AbiReferenceSection.svelte';
	import BlockDefinitionSection from '$lib/components/plugin-builder/BlockDefinitionSection.svelte';
	import ExecutionSection from '$lib/components/plugin-builder/ExecutionSection.svelte';
	import SettingsSchemaSection from '$lib/components/plugin-builder/SettingsSchemaSection.svelte';
	import DependenciesSection from '$lib/components/plugin-builder/DependenciesSection.svelte';
	import BuildSection from '$lib/components/plugin-builder/BuildSection.svelte';
	import OutputPanel from '$lib/components/plugin-builder/OutputPanel.svelte';

	// ── Plugin config state ──
	let pluginName = $state('MyPlugin');
	let pluginVersion = $state('0.1.0');
	let pluginAuthor = $state('');
	let pluginDescription = $state('A custom ironbullet plugin');
	let blocks = $state<PluginBlock[]>([{
		name: 'MyBlock', label: 'My Block', category: 'Utilities', color: '#9b59b6',
		settingsFields: [{ name: 'input_var', type: 'string', default: 'data.SOURCE' }],
	}]);
	let extraDeps = $state<Array<{ name: string; version: string; features: string }>>([]);

	// ── Sidebar navigation ──
	let activeSection = $state<Section>('config');
	const SECTIONS: { id: Section; label: string; icon: SectionIcon }[] = [
		{ id: 'config', label: 'Plugin Config', icon: Settings },
		{ id: 'getting-started', label: 'Getting Started', icon: BookOpen },
		{ id: 'abi-reference', label: 'ABI Reference', icon: Box },
		{ id: 'block-definition', label: 'Block Definition', icon: Puzzle },
		{ id: 'execution', label: 'Execution', icon: Zap },
		{ id: 'settings-schema', label: 'Settings Schema', icon: Settings },
		{ id: 'dependencies', label: 'Dependencies', icon: Package },
		{ id: 'building', label: 'Build & Load', icon: Wrench },
	];

	// ── Build / output state ──
	let outputLines = $state<Array<{ text: string; type: 'info' | 'error' | 'success' | 'cmd' }>>([]);
	let isCompiling = $state(false);
	let showOutput = $state(false);
	let lastBuildSuccess = $state<boolean | null>(null);
	let lastDllPath = $state('');
	let projectDir = $state('');
	let buildRelease = $state(true);
	let breakpoints = $state<Set<number>>(new Set());

	// ── Resizable panels ──
	let centerPanelWidth = $state(360);
	let outputHeight = $state(180);

	function startResize(startVal: number, min: number, max: number, axis: 'x' | 'y', invert: boolean, setter: (v: number) => void) {
		return (e: MouseEvent) => {
			const startPos = axis === 'x' ? e.clientX : e.clientY;
			const onMove = (ev: MouseEvent) => {
				const delta = (axis === 'x' ? ev.clientX : ev.clientY) - startPos;
				setter(Math.max(min, Math.min(max, startVal + (invert ? -delta : delta))));
			};
			const onUp = () => { window.removeEventListener('mousemove', onMove); window.removeEventListener('mouseup', onUp); };
			window.addEventListener('mousemove', onMove);
			window.addEventListener('mouseup', onUp);
		};
	}

	function addOutput(text: string, type: 'info' | 'error' | 'success' | 'cmd' = 'info') {
		outputLines = [...outputLines, { text, type }];
	}
	function clearOutput() { outputLines = []; lastBuildSuccess = null; lastDllPath = ''; }

	// ── Codegen wrappers ──
	function getCargoToml() { return generateCargoToml(pluginName, pluginVersion, extraDeps); }
	function getLibRs() { return generateLibRs(pluginName, pluginVersion, pluginAuthor, pluginDescription, blocks); }

	// ── Actions ──
	function compilePlugin() {
		if (isCompiling) return;
		isCompiling = true; showOutput = true;
		addOutput('', 'cmd'); addOutput('Starting build...', 'info');
		const libCode = editor?.getValue() || getLibRs();
		const cargoCode = cargoEditor?.getValue() || getCargoToml();
		app._compileOutputCallback = (data) => {
			if (data.line) {
				const isErr = data.line.startsWith('error') || data.line.includes('error[');
				const isSuc = data.line.startsWith('Build succeeded');
				addOutput(data.line, isSuc ? 'success' : isErr || data.line.startsWith('warning') ? 'error' : 'info');
			}
			if (data.done) {
				isCompiling = false; lastBuildSuccess = data.success;
				if (data.dll_path) lastDllPath = data.dll_path;
				addOutput(data.success ? 'Build completed successfully.' : 'Build failed.', data.success ? 'success' : 'error');
				app._compileOutputCallback = null;
			}
		};
		send('compile_plugin', { project_dir: projectDir, lib_rs: libCode, cargo_toml: cargoCode, release: buildRelease });
	}

	function debugInspect() {
		showOutput = true; addOutput('', 'cmd');
		addOutput('── Plugin Debug Inspector ──', 'info');
		addOutput(`Plugin: ${pluginName} v${pluginVersion}`, 'info');
		addOutput(`Blocks: ${blocks.length}`, 'info');
		for (const b of blocks) {
			addOutput(`  [${b.category}] ${pluginName}.${b.name} — "${b.label}"`, 'info');
			addOutput(`    Color: ${b.color}  Fields: ${b.settingsFields.length}`, 'info');
			for (const f of b.settingsFields) addOutput(`      ${f.name}: ${f.type} = "${f.default}"`, 'info');
		}
		addOutput(`Extra deps: ${extraDeps.filter(d => d.name).length}`, 'info');
		for (const d of extraDeps.filter(d => d.name))
			addOutput(`  ${d.name} = "${d.version}"${d.features ? ` features=[${d.features}]` : ''}`, 'info');
		if (breakpoints.size > 0)
			addOutput(`Breakpoints set at lines: ${[...breakpoints].sort((a, b) => a - b).join(', ')}`, 'info');
		const issues: string[] = [];
		if (!pluginName.trim()) issues.push('Plugin name is empty');
		for (const b of blocks) {
			if (!b.name.trim()) issues.push(`Block has empty name`);
			if (b.settingsFields.some(f => !f.name.trim())) issues.push(`Block "${b.name}" has field with empty name`);
			const dupes = b.settingsFields.map(f => f.name).filter((n, i, a) => a.indexOf(n) !== i);
			if (dupes.length) issues.push(`Block "${b.name}" has duplicate field names: ${dupes.join(', ')}`);
		}
		if (issues.length) { addOutput('', 'error'); addOutput(`Found ${issues.length} issue(s):`, 'error'); for (const i of issues) addOutput(`  • ${i}`, 'error'); }
		else { addOutput('', 'success'); addOutput('No issues found. Plugin config is valid.', 'success'); }
	}

	function savePluginToDisk() {
		send('save_plugin_files', { lib_rs: editor?.getValue() || getLibRs(), cargo_toml: cargoEditor?.getValue() || getCargoToml(), dir: projectDir });
	}
	function installDll() {
		if (!lastDllPath) { toast('No DLL built yet — compile first', 'warning'); return; }
		send('import_plugin', { path: lastDllPath }); toast('Installing plugin DLL...', 'info');
	}

	// ── Editor state ──
	let editorContainer: HTMLDivElement;
	let cargoContainer: HTMLDivElement;
	let editor = $state<Monaco.editor.IStandaloneCodeEditor | null>(null);
	let cargoEditor = $state<Monaco.editor.IStandaloneCodeEditor | null>(null);
	let monacoRef: typeof Monaco | null = null;
	let activeFile = $state<'lib.rs' | 'Cargo.toml'>('lib.rs');
	let breakpointDecorations = $state<string[]>([]);

	function toggleBreakpoint(lineNumber: number) {
		const next = new Set(breakpoints);
		if (next.has(lineNumber)) next.delete(lineNumber); else next.add(lineNumber);
		breakpoints = next;
		if (!editor || !monacoRef) return;
		const newDecos = [...breakpoints].map(line => ({
			range: new monacoRef!.Range(line, 1, line, 1),
			options: { isWholeLine: true, linesDecorationsClassName: 'breakpoint-glyph', className: 'breakpoint-line-highlight' },
		}));
		breakpointDecorations = editor.deltaDecorations(breakpointDecorations, newDecos);
	}

	function regenerate() {
		editor?.getModel()?.setValue(getLibRs());
		cargoEditor?.getModel()?.setValue(getCargoToml());
	}
	function copyCode() {
		const val = activeFile === 'lib.rs' ? editor?.getValue() : cargoEditor?.getValue();
		if (val) { navigator.clipboard.writeText(val); toast(`${activeFile} copied to clipboard`, 'success'); }
	}
	function downloadPlugin() {
		const dl = (name: string, content: string) => { const b = new Blob([content], { type: 'text/plain' }); const u = URL.createObjectURL(b); const a = document.createElement('a'); a.href = u; a.download = name.replace(/\//g, '_'); a.click(); URL.revokeObjectURL(u); };
		dl('src_lib.rs', editor?.getValue() || getLibRs());
		setTimeout(() => dl('Cargo.toml', cargoEditor?.getValue() || getCargoToml()), 200);
		toast('Plugin files downloaded', 'success');
	}

	function addBlock() { blocks = [...blocks, { name: `Block${blocks.length + 1}`, label: `Block ${blocks.length + 1}`, category: 'Utilities', color: '#9b59b6', settingsFields: [{ name: 'input_var', type: 'string', default: 'data.SOURCE' }] }]; }
	function removeBlock(idx: number) { blocks = blocks.filter((_, i) => i !== idx); }
	function addField(bi: number) { blocks[bi].settingsFields = [...blocks[bi].settingsFields, { name: 'new_field', type: 'string', default: '' }]; }
	function removeField(bi: number, fi: number) { blocks[bi].settingsFields = blocks[bi].settingsFields.filter((_, i) => i !== fi); }
	function addDep() { extraDeps = [...extraDeps, { name: '', version: '', features: '' }]; }
	function removeDep(idx: number) { extraDeps = extraDeps.filter((_, i) => i !== idx); }
	function goBack() { app.showPluginBuilder = false; }

	onMount(async () => {
		const monaco = await import('monaco-editor');
		monacoRef = monaco;
		const EditorWorker = (await import('monaco-editor/esm/vs/editor/editor.worker?worker')).default;
		(self as any).MonacoEnvironment = { getWorker: () => new EditorWorker() };
		try { monaco.editor.defineTheme('ironbullet-dark', { base: 'vs-dark', inherit: true, rules: [{ token: 'keyword', foreground: 'c586c0' }, { token: 'type', foreground: '4ec9b0' }, { token: 'string', foreground: 'ce9178' }, { token: 'number', foreground: 'b5cea8' }, { token: 'comment', foreground: '6a9955', fontStyle: 'italic' }, { token: 'variable', foreground: '9cdcfe' }], colors: { 'editor.background': '#1a1a1d', 'editor.foreground': '#cccccc', 'editor.lineHighlightBackground': '#ffffff06', 'editor.selectionBackground': '#264f78', 'editorCursor.foreground': '#cccccc', 'editorLineNumber.foreground': '#858585', 'editorGutter.background': '#1a1a1d' } }); } catch (_) { /* already defined */ }
		const opts: Monaco.editor.IStandaloneEditorConstructionOptions = { theme: 'ironbullet-dark', minimap: { enabled: false }, fontSize: 12, fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', Consolas, monospace", lineNumbers: 'on', scrollBeyondLastLine: false, automaticLayout: true, tabSize: 4, padding: { top: 8, bottom: 8 }, overviewRulerLanes: 0, hideCursorInOverviewRuler: true, scrollbar: { verticalScrollbarSize: 6, horizontalScrollbarSize: 6 }, folding: true, wordWrap: 'off', matchBrackets: 'always', bracketPairColorization: { enabled: true }, glyphMargin: true };
		editor = monaco.editor.create(editorContainer, { ...opts, value: getLibRs(), language: 'rust' });
		editor.onMouseDown((e) => { if (e.target.type === monaco.editor.MouseTargetType.GUTTER_GLYPH_MARGIN) { const line = e.target.position?.lineNumber; if (line) toggleBreakpoint(line); } });
		cargoEditor = monaco.editor.create(cargoContainer, { ...opts, value: getCargoToml(), language: 'toml' });
	});

	// Zoom: Ctrl+/- scales the entire plugin builder UI, Ctrl+0 resets
	let pluginZoom = $state(1);

	function handleZoomKeydown(e: KeyboardEvent) {
		if (!app.showPluginBuilder) return;
		if (!e.ctrlKey && !e.metaKey) return;
		if (e.key === '=' || e.key === '+') {
			e.preventDefault();
			pluginZoom = Math.min(2.0, Math.round((pluginZoom + 0.1) * 10) / 10);
		} else if (e.key === '-') {
			e.preventDefault();
			pluginZoom = Math.max(0.5, Math.round((pluginZoom - 0.1) * 10) / 10);
		} else if (e.key === '0') {
			e.preventDefault();
			pluginZoom = 1;
		}
	}

	onMount(() => {
		window.addEventListener('keydown', handleZoomKeydown);
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleZoomKeydown);
		if (editor) { editor.dispose(); editor = null; }
		if (cargoEditor) { cargoEditor.dispose(); cargoEditor = null; }
		monacoRef = null; app._compileOutputCallback = null;
	});
</script>

<div class="h-full flex flex-col overflow-hidden bg-surface">
	<!-- Top bar -->
	<div class="flex items-center gap-2 px-3 py-1.5 border-b border-border bg-surface shrink-0">
		<button class="skeu-btn flex items-center gap-1.5 text-[11px] text-muted-foreground" onclick={goBack}><ArrowLeft size={12} />Back</button>
		<div class="w-px h-3.5 bg-border mx-0.5"></div>
		<Puzzle size={14} class="text-muted-foreground" />
		<span class="text-[12px] font-semibold text-foreground">Plugin Builder</span>
		<div class="w-px h-3.5 bg-border mx-0.5"></div>
		<button class="skeu-btn flex items-center gap-1 text-[10px] text-muted-foreground" onclick={regenerate} title="Regenerate code from config"><RefreshCw size={10} />Regen</button>
		<button class="skeu-btn flex items-center gap-1 text-[10px] {isCompiling ? 'opacity-50' : 'text-green'}" onclick={compilePlugin} disabled={isCompiling} title="Compile plugin (cargo build)"><Hammer size={10} />{isCompiling ? 'Building...' : 'Build'}</button>
		<button class="skeu-btn flex items-center gap-1 text-[10px] text-blue-400" onclick={debugInspect} title="Inspect plugin config & validate"><Bug size={10} />Inspect</button>
		{#if lastBuildSuccess && lastDllPath}
			<button class="skeu-btn flex items-center gap-1 text-[10px] text-orange-400" onclick={installDll} title="Install built DLL into plugins"><Play size={10} />Install</button>
		{/if}
		<div class="flex-1"></div>
		{#if pluginZoom !== 1}
			<span class="text-[9px] text-muted-foreground/60 font-mono">{Math.round(pluginZoom * 100)}%</span>
		{/if}
		<button class="skeu-btn flex items-center gap-1 text-[10px] text-muted-foreground" onclick={savePluginToDisk} title="Save plugin project to disk"><FolderOpen size={10} />Save Project</button>
		<button class="skeu-btn flex items-center gap-1 text-[10px] text-muted-foreground" onclick={copyCode}><Copy size={10} />Copy</button>
		<button class="skeu-btn flex items-center gap-1 text-[10px] text-muted-foreground" onclick={downloadPlugin}><Download size={10} />Download</button>
	</div>

	<!-- Main content -->
	<div class="flex flex-1 overflow-hidden" style="zoom: {pluginZoom}">
		<!-- Left sidebar -->
		<div class="w-48 border-r border-border overflow-y-auto shrink-0 bg-surface/50 flex flex-col">
			<div class="py-1">
				{#each SECTIONS as section}
					{@const Icon = section.icon}
					<button class="w-full text-left px-3 py-1.5 text-[11px] hover:bg-accent/30 transition-colors flex items-center gap-2 {activeSection === section.id ? 'bg-accent/40 text-foreground font-medium' : 'text-muted-foreground'}" onclick={() => { activeSection = section.id; }}>
						<Icon size={12} class="shrink-0" /><span class="truncate">{section.label}</span>
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

		<!-- Center: config form or docs -->
		{#if activeSection === 'config'}
			<div class="border-r border-border overflow-y-auto shrink-0" style="width: {centerPanelWidth}px">
				<ConfigSection bind:pluginName bind:pluginVersion bind:pluginAuthor bind:pluginDescription bind:blocks bind:extraDeps bind:projectDir bind:buildRelease onAddBlock={addBlock} onRemoveBlock={removeBlock} onAddField={addField} onRemoveField={removeField} onAddDep={addDep} onRemoveDep={removeDep} />
			</div>
		{:else}
			<div class="border-r border-border overflow-y-auto shrink-0 p-4 guide-content" style="width: {centerPanelWidth}px">
				{#if activeSection === 'getting-started'}<GettingStartedSection />
				{:else if activeSection === 'abi-reference'}<AbiReferenceSection />
				{:else if activeSection === 'block-definition'}<BlockDefinitionSection />
				{:else if activeSection === 'execution'}<ExecutionSection />
				{:else if activeSection === 'settings-schema'}<SettingsSchemaSection />
				{:else if activeSection === 'dependencies'}<DependenciesSection />
				{:else if activeSection === 'building'}<BuildSection />
				{/if}
			</div>
		{/if}

		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="groove-v resize-handle w-[3px] shrink-0" onmousedown={startResize(centerPanelWidth, 260, 550, 'x', false, v => centerPanelWidth = v)}></div>

		<!-- Right: Code editor + output -->
		<div class="flex-1 flex flex-col min-w-0">
			<div class="flex items-center gap-0 px-1 py-0.5 bg-surface border-b border-border shrink-0">
				<button class="px-3 py-1 text-[11px] rounded-t transition-colors {activeFile === 'lib.rs' ? 'bg-[#1a1a1d] text-foreground font-medium border border-border border-b-transparent -mb-px' : 'text-muted-foreground hover:text-foreground'}" onclick={() => { activeFile = 'lib.rs'; }}>src/lib.rs</button>
				<button class="px-3 py-1 text-[11px] rounded-t transition-colors {activeFile === 'Cargo.toml' ? 'bg-[#1a1a1d] text-foreground font-medium border border-border border-b-transparent -mb-px' : 'text-muted-foreground hover:text-foreground'}" onclick={() => { activeFile = 'Cargo.toml'; }}>Cargo.toml</button>
				<div class="flex-1"></div>
				{#if breakpoints.size > 0}
					<div class="flex items-center gap-1 text-[9px] text-orange-400 mr-2"><CircleDot size={9} />{breakpoints.size} breakpoint{breakpoints.size > 1 ? 's' : ''}</div>
				{/if}
				<button class="p-1 rounded hover:bg-accent/30 text-muted-foreground" onclick={() => { showOutput = !showOutput; }} title={showOutput ? 'Hide output' : 'Show output'}><Terminal size={12} /></button>
			</div>
			<div class="flex-1 overflow-hidden" style="display: {activeFile === 'lib.rs' ? 'block' : 'none'}" bind:this={editorContainer}></div>
			<div class="flex-1 overflow-hidden" style="display: {activeFile === 'Cargo.toml' ? 'block' : 'none'}" bind:this={cargoContainer}></div>
			{#if showOutput}
				<OutputPanel {outputLines} {isCompiling} {lastBuildSuccess} {outputHeight} onClear={clearOutput} onClose={() => { showOutput = false; }} onResizeStart={startResize(outputHeight, 80, 400, 'y', true, v => outputHeight = v)} />
			{/if}
		</div>
	</div>
</div>

<style>
	.guide-content :global(h3) { color: var(--foreground); }
	.guide-content :global(p) { color: var(--muted-foreground); overflow-wrap: break-word; word-break: break-word; }
	.guide-content :global(li) { color: var(--foreground); opacity: 0.85; overflow-wrap: break-word; word-break: break-word; }
	.guide-content :global(code) { background: var(--accent); padding: 1px 4px; border-radius: 3px; font-size: 10px; font-family: var(--font-mono, monospace); word-break: break-all; }
	.guide-content :global(pre) { color: var(--foreground); opacity: 0.9; overflow-x: auto; white-space: pre; max-width: 100%; user-select: text; }
	.guide-content :global(pre code) { background: none; padding: 0; }
	.guide-content :global(table) { border: 1px solid var(--border); border-radius: 6px; overflow: hidden; table-layout: fixed; width: 100%; word-break: break-word; }
	.guide-content :global(td), .guide-content :global(th) { color: var(--foreground); overflow-wrap: break-word; }
	.guide-content :global(kbd) { background: var(--accent); padding: 1px 6px; border-radius: 3px; border: 1px solid var(--border); font-size: 10px; font-family: var(--font-mono, monospace); }
	.guide-content :global(ol) { list-style-type: decimal; }
	.guide-content :global(ul) { list-style-type: disc; }
	.guide-content { user-select: text; }
	:global(.breakpoint-glyph) { background: #e53935; border-radius: 50%; width: 8px !important; height: 8px !important; margin-left: 4px; margin-top: 4px; }
	:global(.breakpoint-line-highlight) { background: rgba(229, 57, 53, 0.08) !important; }
</style>
