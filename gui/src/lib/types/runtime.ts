import type { BlockType } from './pipeline';

export interface RunnerStats {
	total: number;
	processed: number;
	hits: number;
	fails: number;
	bans: number;
	retries: number;
	errors: number;
	cpm: number;
	active_threads: number;
	elapsed_secs: number;
}

export interface BlockResult {
	block_id: string;
	block_label: string;
	block_type: BlockType;
	success: boolean;
	timing_ms: number;
	variables_after: Record<string, string>;
	log_message: string;
	request?: RequestInfo;
	response?: ResponseInfo;
}

export interface RequestInfo {
	method: string;
	url: string;
	headers: [string, string][];
	body: string;
}

export interface ResponseInfo {
	status_code: number;
	headers: Record<string, string>;
	body: string;
	final_url: string;
	cookies: Record<string, string>;
	timing_ms: number;
}

export interface NetworkEntry {
	block_id: string;
	block_label: string;
	method: string;
	url: string;
	status_code: number;
	timing_ms: number;
	response_size: number;
	cookies_set: [string, string][];
	cookies_sent: [string, string][];
}
