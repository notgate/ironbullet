/**
 * Global singleton state for the intellisense popup.
 *
 * VariableInput writes here when it wants to show suggestions.
 * IntelliPopup.svelte (mounted once at app root) reads and renders it.
 * This avoids any z-index / overflow-clip / container stacking context issues.
 */
import type { SuggestionItem } from './components/Intellisense.svelte';

interface PopupState {
	visible: boolean;
	suggestions: SuggestionItem[];
	anchorRect: DOMRect | null;
	selectedIndex: number;
	onpick: ((item: SuggestionItem) => void) | null;
	onclose: (() => void) | null;
}

function createPopupStore() {
	let state = $state<PopupState>({
		visible: false,
		suggestions: [],
		anchorRect: null,
		selectedIndex: 0,
		onpick: null,
		onclose: null,
	});

	return {
		get visible()       { return state.visible; },
		get suggestions()   { return state.suggestions; },
		get anchorRect()    { return state.anchorRect; },
		get selectedIndex() { return state.selectedIndex; },

		show(
			suggestions: SuggestionItem[],
			anchorRect: DOMRect,
			onpick: (item: SuggestionItem) => void,
			onclose: () => void,
		) {
			state.visible       = true;
			state.suggestions   = suggestions;
			state.anchorRect    = anchorRect;
			state.selectedIndex = 0;
			state.onpick        = onpick;
			state.onclose       = onclose;
		},

		update(suggestions: SuggestionItem[], anchorRect: DOMRect) {
			if (!state.visible) return;
			state.suggestions = suggestions;
			state.anchorRect  = anchorRect;
			// reset selection only if suggestion list changed length meaningfully
			if (state.selectedIndex >= suggestions.length) state.selectedIndex = 0;
		},

		hide() {
			state.visible = false;
			state.onclose?.();
		},

		pick(item?: SuggestionItem) {
			const target = item ?? state.suggestions[state.selectedIndex];
			if (target) {
				state.onpick?.(target);
				state.visible = false;
			}
		},

		moveUp() {
			if (!state.visible || state.suggestions.length === 0) return;
			state.selectedIndex = (state.selectedIndex - 1 + state.suggestions.length) % state.suggestions.length;
		},

		moveDown() {
			if (!state.visible || state.suggestions.length === 0) return;
			state.selectedIndex = (state.selectedIndex + 1) % state.suggestions.length;
		},

		setHover(i: number) {
			state.selectedIndex = i;
		},
	};
}

export const intelliPopup = createPopupStore();
