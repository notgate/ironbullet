<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';

	let show = $state(false);
	let step = $state(0);

	// Dynamic layout values from app state
	let lpw = $derived(app.showBlockPalette ? app.leftPanelWidth : 0);
	let bph = $derived(app.bottomPanelHeight);
	const TOOLBAR_H = 40;

	// Show onboarding only on first launch (no pipeline loaded, no recent configs)
	$effect(() => {
		if (!app.showStartup && app.pipeline.blocks.length === 0 && !localStorage.getItem('rf_onboarding_done')) {
			show = true;
		}
	});

	const STEPS = [
		{
			title: 'Welcome to reqflow',
			body: 'A visual pipeline builder for HTTP automation. Let\'s take a quick tour of the interface.',
			highlight: null,
		},
		{
			title: 'Block Palette',
			body: 'Drag blocks from here into the pipeline editor. Blocks are organized by category: Request, Parse, Check, Function, and more.',
			highlight: 'left',
		},
		{
			title: 'Pipeline Editor',
			body: 'This is your workspace. Blocks execute top-to-bottom. Click a block to configure it in the side panel.',
			highlight: 'center',
		},
		{
			title: 'Block Settings',
			body: 'When you click a block, its settings appear in a slide-in panel on the right. Each field has a description to help you.',
			highlight: 'right',
		},
		{
			title: 'Bottom Panel',
			body: 'Debug your pipeline, view generated code, manage data/proxies, and inspect variables. Use the tabs to switch views.',
			highlight: 'bottom',
		},
		{
			title: 'Toolbar',
			body: 'Use File menu to save/load configs, Run to debug or start the runner, and the quick action buttons for one-click access.',
			highlight: 'top',
		},
	];

	// Compute arrow position relative to actual panel dimensions
	function arrowPos(h: string | null): { x: number; y: number; angle: number; label: string } | null {
		if (!h) return null;
		const winW = typeof window !== 'undefined' ? window.innerWidth : 900;
		const winH = typeof window !== 'undefined' ? window.innerHeight : 600;
		switch (h) {
			case 'left': return { x: Math.max(30, lpw / 2 - 30), y: 200, angle: -135, label: 'Drag blocks from here' };
			case 'center': return { x: lpw + 160, y: 180, angle: -90, label: 'Blocks go here' };
			case 'right': return { x: Math.max(500, winW - 200), y: 200, angle: -45, label: 'Configure here' };
			case 'bottom': return { x: lpw + 160, y: winH - bph - 40, angle: 90, label: 'Debug & run tools' };
			case 'top': return { x: 200, y: TOOLBAR_H + 15, angle: 180, label: 'Menus & actions' };
			default: return null;
		}
	}

	function next() {
		if (step < STEPS.length - 1) {
			step++;
		} else {
			finish();
		}
	}

	function prev() {
		if (step > 0) step--;
	}

	function finish() {
		show = false;
		localStorage.setItem('rf_onboarding_done', '1');
	}

	function skip() {
		finish();
	}
</script>

{#if show}
	{@const hl = STEPS[step].highlight}
	{@const arrow = arrowPos(hl)}
	<!-- Full-screen overlay -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="fixed inset-0 top-[28px] z-[9998] onboarding-overlay" onclick={(e) => e.stopPropagation()}>
		<!-- Dim background -->
		<div class="absolute inset-0 bg-black/60"></div>

		<!-- Highlight regions — computed from live panel sizes -->
		{#if hl === 'left'}
			<div class="absolute highlight-pulse" style="top: {TOOLBAR_H}px; left: 0; bottom: 0; width: {lpw}px;"></div>
		{:else if hl === 'center'}
			<div class="absolute highlight-pulse" style="top: {TOOLBAR_H}px; left: {lpw + 3}px; bottom: {bph}px; right: 0;"></div>
		{:else if hl === 'right'}
			<div class="absolute highlight-pulse" style="top: {TOOLBAR_H}px; right: 0; bottom: {bph}px; width: 360px;"></div>
		{:else if hl === 'bottom'}
			<div class="absolute highlight-pulse" style="bottom: 0; left: 0; right: 0; height: {bph}px;"></div>
		{:else if hl === 'top'}
			<div class="absolute highlight-pulse" style="top: 0; left: 0; right: 0; height: {TOOLBAR_H}px;"></div>
		{/if}

		<!-- Animated arrow doodle -->
		{#if arrow}
			<div class="absolute arrow-doodle" style="left: {arrow.x}px; top: {arrow.y}px;">
				<svg width="120" height="60" viewBox="0 0 120 60" class="arrow-svg" style="transform: rotate({arrow.angle}deg);">
					<path d="M 10 30 Q 40 10 60 25 Q 80 40 100 20" stroke="var(--primary)" stroke-width="2.5" fill="none" stroke-dasharray="5,5" class="arrow-path" />
					<polygon points="95,12 105,20 93,24" fill="var(--primary)" class="arrow-head" />
				</svg>
				<span class="arrow-label">{arrow.label}</span>
			</div>
		{/if}

		<!-- Step card -->
		<div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 onboarding-card">
			<div class="bg-surface border border-border rounded-lg p-5 max-w-[380px] shadow-2xl">
				<!-- Step indicator -->
				<div class="flex gap-1 mb-3">
					{#each STEPS as _, i}
						<div class="h-1 flex-1 rounded-full transition-colors {i <= step ? 'bg-primary' : 'bg-border'}"></div>
					{/each}
				</div>

				<h3 class="text-[14px] font-semibold text-foreground mb-1.5">{STEPS[step].title}</h3>
				<p class="text-[12px] text-muted-foreground leading-relaxed mb-4">{STEPS[step].body}</p>

				<div class="flex items-center justify-between">
					<button class="text-[11px] text-muted-foreground hover:text-foreground" onclick={skip}>
						Skip tour
					</button>
					<div class="flex gap-2">
						{#if step > 0}
							<button class="skeu-btn text-[11px]" onclick={prev}>Back</button>
						{/if}
						<button
							class="skeu-btn text-[11px] bg-primary/20 text-primary hover:bg-primary/30"
							onclick={next}
						>
							{step === STEPS.length - 1 ? 'Get Started' : 'Next'}
							{#if step < STEPS.length - 1}<ArrowRight size={11} class="inline ml-1" />{/if}
						</button>
					</div>
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.onboarding-overlay {
		animation: fadeIn 0.3s ease-out;
	}

	@keyframes fadeIn {
		from { opacity: 0; }
		to { opacity: 1; }
	}

	.highlight-pulse {
		border: 2px solid var(--primary);
		border-radius: 4px;
		animation: highlightPulse 2s ease-in-out infinite;
		pointer-events: none;
		z-index: 1;
	}

	@keyframes highlightPulse {
		0%, 100% { border-color: rgba(0, 120, 212, 0.3); background: rgba(0, 120, 212, 0.03); }
		50% { border-color: rgba(0, 120, 212, 0.7); background: rgba(0, 120, 212, 0.08); }
	}

	.onboarding-card {
		z-index: 2;
		animation: cardSlideIn 0.3s ease-out;
	}

	@keyframes cardSlideIn {
		from { transform: translate(-50%, -50%) scale(0.95); opacity: 0; }
		to { transform: translate(-50%, -50%) scale(1); opacity: 1; }
	}

	.arrow-doodle {
		z-index: 2;
		pointer-events: none;
		animation: arrowBounce 2s ease-in-out infinite;
	}

	@keyframes arrowBounce {
		0%, 100% { transform: translateY(0); }
		50% { transform: translateY(-8px); }
	}

	.arrow-path {
		animation: arrowDash 1.5s linear infinite;
	}

	@keyframes arrowDash {
		from { stroke-dashoffset: 0; }
		to { stroke-dashoffset: 20; }
	}

	.arrow-head {
		animation: arrowHeadPulse 1.5s ease-in-out infinite;
	}

	@keyframes arrowHeadPulse {
		0%, 100% { opacity: 0.7; }
		50% { opacity: 1; }
	}

	.arrow-label {
		position: absolute;
		bottom: -18px;
		left: 50%;
		transform: translateX(-50%);
		white-space: nowrap;
		font-size: 10px;
		color: var(--primary);
		font-weight: 500;
		text-shadow: 0 1px 3px rgba(0,0,0,0.5);
	}
</style>
