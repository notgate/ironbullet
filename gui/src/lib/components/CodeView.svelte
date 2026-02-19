<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import { toast } from '$lib/toast.svelte';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import Copy from '@lucide/svelte/icons/copy';
	import Save from '@lucide/svelte/icons/save';
	import type * as Monaco from 'monaco-editor';

	let container: HTMLDivElement;
	let editor = $state<Monaco.editor.IStandaloneCodeEditor | null>(null);
	let monacoRef: typeof Monaco | null = null;
	let isEdited = $state(false);
	let settingProgrammatically = false;
	let code = $derived(app.generatedCode);

	// Sync external code changes into editor (when not manually edited)
	$effect(() => {
		if (editor && code && !isEdited) {
			const model = editor.getModel();
			if (model && model.getValue() !== code) {
				settingProgrammatically = true;
				model.setValue(code);
				settingProgrammatically = false;
			}
		}
	});

	// Auto-refresh when pipeline changes (deep watch via serialization)
	// Debounced to prevent UI freezes when clicking blocks rapidly
	let pipelineHash = $derived(JSON.stringify(app.pipeline));
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		void pipelineHash;
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			send('generate_code', { pipeline: app.pipeline });
		}, 300); // Wait 300ms after last change before regenerating
		return () => {
			if (debounceTimer) clearTimeout(debounceTimer);
		};
	});

	function refreshCode() {
		isEdited = false;
		send('generate_code', { pipeline: app.pipeline });
	}

	function copyCode() {
		const val = editor?.getValue() || code;
		navigator.clipboard.writeText(val);
		toast('Code copied to clipboard', 'success');
	}

	function saveCode() {
		const val = editor?.getValue() || code;
		send('save_code', { code: val });
	}

	onMount(async () => {
		const monaco = await import('monaco-editor');
		monacoRef = monaco;

		// Set up Monaco web worker
		const EditorWorker = (await import('monaco-editor/esm/vs/editor/editor.worker?worker')).default;
		(self as any).MonacoEnvironment = {
			getWorker: () => new EditorWorker()
		};

		// Custom dark theme matching our skeuomorphic UI
		monaco.editor.defineTheme('ironbullet-dark', {
			base: 'vs-dark',
			inherit: true,
			rules: [
				{ token: 'keyword', foreground: 'c586c0' },
				{ token: 'keyword.control', foreground: 'c586c0' },
				{ token: 'type', foreground: '4ec9b0' },
				{ token: 'type.identifier', foreground: '4ec9b0' },
				{ token: 'entity.name.function', foreground: 'dcdcaa' },
				{ token: 'string', foreground: 'ce9178' },
				{ token: 'string.quoted', foreground: 'ce9178' },
				{ token: 'number', foreground: 'b5cea8' },
				{ token: 'comment', foreground: '6a9955', fontStyle: 'italic' },
				{ token: 'attribute.name', foreground: '9cdcfe' },
				{ token: 'variable', foreground: '9cdcfe' },
				{ token: 'constant', foreground: '569cd6' },
			],
			colors: {
				'editor.background': '#1a1a1d',
				'editor.foreground': '#cccccc',
				'editor.lineHighlightBackground': '#ffffff06',
				'editor.selectionBackground': '#264f78',
				'editorCursor.foreground': '#cccccc',
				'editorLineNumber.foreground': '#858585',
				'editorLineNumber.activeForeground': '#cccccc',
				'editor.inactiveSelectionBackground': '#264f7840',
				'editorIndentGuide.background1': '#3e3e44',
				'editorWidget.background': '#222225',
				'editorWidget.border': '#3e3e44',
				'input.background': '#2e2e33',
				'input.border': '#3e3e44',
				'focusBorder': '#0078d4',
				'editorGutter.background': '#1a1a1d',
				'editorOverviewRuler.border': '#1a1a1d',
			},
		});

		editor = monaco.editor.create(container, {
			value: code || '// No blocks in pipeline\n// Add blocks to generate code',
			language: 'rust',
			theme: 'ironbullet-dark',
			minimap: { enabled: false },
			fontSize: 12,
			fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', Consolas, monospace",
			lineNumbers: 'on',
			scrollBeyondLastLine: false,
			automaticLayout: true,
			tabSize: 4,
			renderWhitespace: 'none',
			padding: { top: 8, bottom: 8 },
			overviewRulerLanes: 0,
			hideCursorInOverviewRuler: true,
			scrollbar: {
				verticalScrollbarSize: 6,
				horizontalScrollbarSize: 6,
			},
			contextmenu: true,
			folding: true,
			wordWrap: 'off',
			matchBrackets: 'always',
			bracketPairColorization: { enabled: true },
		});

		// Track manual edits (ignore programmatic setValue calls)
		editor.onDidChangeModelContent(() => {
			if (!settingProgrammatically) {
				isEdited = true;
			}
		});

		// Ctrl+S to save
		editor.addAction({
			id: 'save-code',
			label: 'Save Code',
			keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS],
			run: () => saveCode(),
		});
	});

	onDestroy(() => {
		if (editor) {
			editor.dispose();
			editor = null;
		}
		monacoRef = null;
	});
</script>

<div class="flex flex-col h-full bg-surface">
	<!-- Toolbar -->
	<div class="flex items-center gap-2 px-2 py-1 panel-raised">
		<span class="text-xs text-muted-foreground flex-1">
			Generated Rust Code
			{#if isEdited}<span class="text-yellow text-[9px] ml-1">(edited)</span>{/if}
		</span>
		<button
			class="skeu-btn flex items-center gap-1 text-[10px] text-muted-foreground"
			onclick={refreshCode}
		><RefreshCw size={10} />Refresh</button>
		<button
			class="skeu-btn flex items-center gap-1 text-[10px] text-muted-foreground"
			onclick={copyCode}
		><Copy size={10} />Copy</button>
		<button
			class="skeu-btn flex items-center gap-1 text-[10px] text-green"
			onclick={saveCode}
		><Save size={10} />Save</button>
	</div>

	<!-- Monaco editor container -->
	<div class="flex-1 overflow-hidden" bind:this={container}></div>
</div>
