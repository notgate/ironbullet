export type { BlockDoc, GuideSection } from './types';

import { REQUEST_DOCS } from './requests';
import { PARSING_DOCS } from './parsing';
import { CHECK_DOCS } from './checks';
import { FUNCTION_DOCS } from './functions';
import { CONTROL_DOCS } from './control';
import { UTILITY_DOCS } from './utilities';
import { BYPASS_DOCS } from './bypass';
import { BROWSER_DOCS } from './browser';

export { GUIDE_SECTIONS } from './guides';

export const BLOCK_DOCS_FULL = [
	...REQUEST_DOCS,
	...PARSING_DOCS,
	...CHECK_DOCS,
	...FUNCTION_DOCS,
	...CONTROL_DOCS,
	...UTILITY_DOCS,
	...BYPASS_DOCS,
	...BROWSER_DOCS,
];

export {
	REQUEST_DOCS,
	PARSING_DOCS,
	CHECK_DOCS,
	FUNCTION_DOCS,
	CONTROL_DOCS,
	UTILITY_DOCS,
	BYPASS_DOCS,
	BROWSER_DOCS,
};
