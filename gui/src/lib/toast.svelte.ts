export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface ToastItem {
	id: number;
	message: string;
	type: ToastType;
	leaving: boolean;
}

const DURATION: Record<ToastType, number> = {
	success: 3000,
	info: 4000,
	warning: 5000,
	error: 6000,
};

let _toasts = $state<ToastItem[]>([]);
let _nextId = 0;

export function getToasts(): ToastItem[] {
	return _toasts;
}

export function toast(message: string, type: ToastType = 'info') {
	const id = _nextId++;
	_toasts = [..._toasts, { id, message, type, leaving: false }];
	setTimeout(() => dismiss(id), DURATION[type]);
}

export function dismiss(id: number) {
	_toasts = _toasts.map(t => t.id === id ? { ...t, leaving: true } : t);
	setTimeout(() => {
		_toasts = _toasts.filter(t => t.id !== id);
	}, 250);
}
