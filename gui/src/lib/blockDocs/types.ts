export interface BlockDoc {
	type: string;
	name: string;
	category: string;
	description: string;
	parameters: Array<{
		name: string;
		type: string;
		required: boolean;
		description: string;
		default?: string;
	}>;
	codeExample: string;
	tips: string[];
	relatedBlocks: string[];
	rustCode?: string;
}

export interface GuideSection {
	id: string;
	title: string;
	icon: string;
	content: string;
}
