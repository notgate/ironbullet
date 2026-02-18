import type BookOpen from '@lucide/svelte/icons/book-open';

export interface PluginBlock {
	name: string;
	label: string;
	category: string;
	color: string;
	settingsFields: Array<{ name: string; type: string; default: string }>;
}

export type Section = 'config' | 'getting-started' | 'abi-reference' | 'block-definition' | 'execution' | 'settings-schema' | 'dependencies' | 'building';

export const CATEGORIES = ['Requests', 'Parsing', 'Checks', 'Functions', 'Control', 'Utilities', 'Bypass', 'Sensors', 'Browser'];

export const FIELD_TYPES = [
	{ value: 'string', label: 'str' },
	{ value: 'number', label: 'num' },
	{ value: 'boolean', label: 'bool' },
];

export type SectionIcon = typeof BookOpen;
