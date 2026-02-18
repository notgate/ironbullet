import type { BlockType, Pipeline } from './pipeline';

// Config tab for multi-config management
export interface ConfigTab {
	id: string;
	name: string;
	filePath: string | null;
	pipeline: Pipeline;
	isDirty: boolean;
	savedSnapshot: string; // JSON snapshot for dirty comparison
}

// Block metadata for palette
export interface BlockMeta {
	type: BlockType;
	label: string;
	category: string;
	color: string;
	icon: string;
}

// Plugin metadata
export interface PluginBlockMeta {
	block_type_name: string;
	label: string;
	category: string;
	color: string;
	icon: string;
	settings_schema_json: string;
	default_settings_json: string;
	plugin_name: string;
	block_index: number;
}

export interface PluginMeta {
	name: string;
	version: string;
	author: string;
	description: string;
	dll_path: string;
}
