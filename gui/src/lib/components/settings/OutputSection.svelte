<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import SkeuSelect from '$lib/components/SkeuSelect.svelte';

	let { searchQuery, shouldShowSetting }: {
		searchQuery: string;
		shouldShowSetting: (section: string, label: string) => boolean;
	} = $props();
</script>

{#if searchQuery}<div class="text-[10px] uppercase tracking-widest text-muted-foreground/70 font-semibold mb-2 mt-2">Output</div>{/if}

{#if shouldShowSetting('output', 'Save to File')}
	<div class="flex items-center justify-between py-1.5">
		<div>
			<span class="text-[11px] text-muted-foreground">Save to text files</span>
			<p class="text-[9px] text-muted-foreground/60">Write hits to .txt files per status</p>
		</div>
		<!-- svelte-ignore a11y_label_has_associated_control -->
		<label class="flex items-center gap-2 cursor-pointer">
			<input
				type="checkbox"
				checked={app.pipeline.output_settings.save_to_file}
				onchange={() => { app.pipeline.output_settings.save_to_file = !app.pipeline.output_settings.save_to_file; }}
				class="skeu-checkbox"
			/>
		</label>
	</div>
{/if}

{#if shouldShowSetting('output', 'Save to Database')}
	<div class="flex items-center justify-between py-1.5">
		<div>
			<span class="text-[11px] text-muted-foreground">Save to database</span>
			<p class="text-[9px] text-muted-foreground/60">Store results in SQLite database</p>
		</div>
		<!-- svelte-ignore a11y_label_has_associated_control -->
		<label class="flex items-center gap-2 cursor-pointer">
			<input
				type="checkbox"
				checked={app.pipeline.output_settings.save_to_database}
				onchange={() => { app.pipeline.output_settings.save_to_database = !app.pipeline.output_settings.save_to_database; }}
				class="skeu-checkbox"
			/>
		</label>
	</div>
	{#if app.pipeline.output_settings.save_to_database}
		<div class="flex items-center justify-between py-1 pl-4">
			<span class="text-[10px] text-muted-foreground/80">Database path</span>
			<input
				type="text"
				bind:value={app.pipeline.output_settings.database_path}
				class="w-36 skeu-input text-[10px]"
				placeholder="results.db"
			/>
		</div>
	{/if}
{/if}

{#if shouldShowSetting('output', 'Include Response')}
	<div class="flex items-center justify-between py-1.5">
		<div>
			<span class="text-[11px] text-muted-foreground">Include response (-d)</span>
			<p class="text-[9px] text-muted-foreground/60">Append full response body in output</p>
		</div>
		<!-- svelte-ignore a11y_label_has_associated_control -->
		<label class="flex items-center gap-2 cursor-pointer">
			<input
				type="checkbox"
				checked={app.pipeline.output_settings.include_response}
				onchange={() => { app.pipeline.output_settings.include_response = !app.pipeline.output_settings.include_response; }}
				class="skeu-checkbox"
			/>
		</label>
	</div>
{/if}

{#if shouldShowSetting('output', 'Output Directory')}
	<div class="py-1.5">
		<div class="flex items-center justify-between mb-0.5">
			<div>
				<span class="text-[11px] text-muted-foreground">Output directory</span>
				<p class="text-[9px] text-muted-foreground/60">Folder for result text files</p>
			</div>
			<button class="skeu-btn text-[10px]" onclick={() => send('browse_folder', { field: 'results' })}>Browse</button>
		</div>
		<input
			type="text"
			bind:value={app.pipeline.output_settings.output_directory}
			class="w-full skeu-input text-[10px] font-mono"
			placeholder="results/"
		/>
	</div>
{/if}

{#if shouldShowSetting('output', 'Output Format')}
	<div class="py-1.5">
		<span class="text-[11px] text-muted-foreground">Output format</span>
		<p class="text-[9px] text-muted-foreground/60 mb-1">Template: {'{data}'}, {'{captures}'}, {'{status}'}, {'{response}'}</p>
		<input
			type="text"
			bind:value={app.pipeline.output_settings.output_format}
			class="w-full skeu-input text-[10px]"
			placeholder="{'{data}'} | {'{captures}'}"
		/>
	</div>

	<!-- Output Format Type -->
	<div class="flex items-center justify-between">
		<span class="text-[11px]">File Format</span>
		<SkeuSelect
			value={app.pipeline.output_settings.output_format_type}
			onValueChange={(v) => { app.pipeline.output_settings.output_format_type = v as any; }}
			options={[{value:'Txt',label:'TXT'},{value:'Csv',label:'CSV'},{value:'Json',label:'JSON'}]}
			class="text-[10px] w-24"
		/>
	</div>

	<!-- Capture Filters -->
	<div>
		<div class="flex items-center justify-between mb-1">
			<span class="text-[11px]">Capture Filters</span>
			<button class="skeu-btn text-[9px] px-2 py-0.5" onclick={() => {
				app.pipeline.output_settings.capture_filters = [
					...app.pipeline.output_settings.capture_filters,
					{ variable_name: '*', filter_type: 'NotEmpty', value: '', negate: false }
				];
			}}>+ Add</button>
		</div>
		{#each app.pipeline.output_settings.capture_filters as filter, i}
			<div class="flex gap-1 items-center mb-1">
				<input
					type="text"
					bind:value={filter.variable_name}
					class="skeu-input text-[9px] w-20"
					placeholder="* or var name"
				/>
				<SkeuSelect
					value={filter.filter_type}
					onValueChange={(v) => { filter.filter_type = v as any; }}
					options={[
						{value:'Contains',label:'Contains'},{value:'Equals',label:'Equals'},
						{value:'StartsWith',label:'Starts With'},{value:'EndsWith',label:'Ends With'},
						{value:'MatchesRegex',label:'Regex'},{value:'MinLength',label:'Min Length'},
						{value:'MaxLength',label:'Max Length'},{value:'NotEmpty',label:'Not Empty'},
					]}
					class="text-[9px] w-24"
				/>
				<input
					type="text"
					bind:value={filter.value}
					class="skeu-input text-[9px] w-16"
					placeholder="value"
				/>
				<label class="flex items-center gap-1 text-[9px]">
					<input type="checkbox" bind:checked={filter.negate} class="skeu-checkbox" />
					<span>Negate</span>
				</label>
				<button class="text-[9px] text-red-400 hover:text-red-300 px-1" onclick={() => {
					app.pipeline.output_settings.capture_filters = app.pipeline.output_settings.capture_filters.filter((_, idx) => idx !== i);
				}}>x</button>
			</div>
		{/each}
	</div>
{/if}

{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}
