import { app } from './app.svelte';

export function zoomIn() {
	app.zoom = Math.min(2.0, Math.round((app.zoom + 0.1) * 10) / 10);
}

export function zoomOut() {
	app.zoom = Math.max(0.5, Math.round((app.zoom - 0.1) * 10) / 10);
}

export function zoomReset() {
	app.zoom = 1.0;
}
