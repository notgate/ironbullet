/**
 * Svelte use:intellisense action — attaches intellisense to any <input> or <textarea>.
 *
 * Usage:
 *   <textarea use:intellisense={{ context: 'generic', responseBody: ... }} ... />
 *
 * The action wires keydown / input / blur events and pushes to the global
 * intelliPopup store. No extra wrapper element needed.
 */
import { intelliPopup } from './intellisense-popup.svelte';
import { buildSuggestions, getQueryAtCursor, applySuggestion } from './intellisense';
import type { FieldContext } from './intellisense';
import { app } from './state.svelte';
import type { SuggestionItem } from './components/Intellisense.svelte';

export interface IntellisenseOptions {
	context?: FieldContext;
	responseBody?: string;
}

export function intellisense(
	node: HTMLInputElement | HTMLTextAreaElement,
	opts: IntellisenseOptions = {},
) {
	let isOwner = false;
	let options = opts;

	function getRect(): DOMRect {
		return node.getBoundingClientRect();
	}

	function refresh() {
		if (!app.uiPrefs.intellisenseEnabled) {
			if (isOwner) { intelliPopup.hide(); isOwner = false; }
			return;
		}
		const pos = node.selectionStart ?? 0;
		const val = node.value ?? '';
		const trigger = getQueryAtCursor(val, pos);
		if (!trigger) {
			if (isOwner) { intelliPopup.hide(); isOwner = false; }
			return;
		}
		const sug = buildSuggestions(
			options.context ?? 'generic',
			trigger.query,
			app.pipeline,
			options.responseBody,
			val,
			pos,
		);
		if (sug.length === 0) {
			if (isOwner) { intelliPopup.hide(); isOwner = false; }
			return;
		}
		const rect = getRect();
		if (isOwner && intelliPopup.visible) {
			intelliPopup.update(sug, rect);
		} else {
			isOwner = true;
			intelliPopup.show(sug, rect, handlePick, () => { isOwner = false; });
		}
	}

	function handlePick(item: SuggestionItem) {
		const pos = node.selectionStart ?? node.value.length;
		const { newValue, newCursor } = applySuggestion(node.value ?? '', pos, item);

		// Update the native element value and fire an input event so Svelte bindings update
		const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
			node instanceof HTMLTextAreaElement
				? HTMLTextAreaElement.prototype
				: HTMLInputElement.prototype,
			'value',
		)?.set;
		nativeInputValueSetter?.call(node, newValue);
		node.dispatchEvent(new Event('input', { bubbles: true }));

		isOwner = false;
		requestAnimationFrame(() => {
			node.setSelectionRange(newCursor, newCursor);
			node.focus();
		});
	}

	function onKeydown(e: KeyboardEvent) {
		if (!isOwner || !intelliPopup.visible) return;
		if (e.key === 'ArrowDown') { e.preventDefault(); intelliPopup.moveDown(); return; }
		if (e.key === 'ArrowUp')   { e.preventDefault(); intelliPopup.moveUp();   return; }
		if (e.key === 'Tab' || e.key === 'Enter') {
			// For textareas we only intercept Tab (Enter = newline is natural)
			if (e.key === 'Enter' && node instanceof HTMLTextAreaElement) return;
			e.preventDefault();
			intelliPopup.pick();
			return;
		}
		if (e.key === 'Escape') { intelliPopup.hide(); isOwner = false; return; }
	}

	function onInput() { refresh(); }
	function onClick() { refresh(); }
	function onBlur() {
		setTimeout(() => {
			if (isOwner) { intelliPopup.hide(); isOwner = false; }
		}, 150);
	}

	node.addEventListener('keydown', onKeydown);
	node.addEventListener('input', onInput);
	node.addEventListener('click', onClick);
	node.addEventListener('blur', onBlur);

	return {
		update(newOpts: IntellisenseOptions) {
			options = newOpts;
		},
		destroy() {
			node.removeEventListener('keydown', onKeydown);
			node.removeEventListener('input', onInput);
			node.removeEventListener('click', onClick);
			node.removeEventListener('blur', onBlur);
			if (isOwner) { intelliPopup.hide(); isOwner = false; }
		},
	};
}
