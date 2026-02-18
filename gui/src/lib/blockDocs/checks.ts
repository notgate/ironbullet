import type { BlockDoc } from './types';

export const CHECK_DOCS: BlockDoc[] = [
	{
		type: 'KeyCheck',
		name: 'Key Check',
		category: 'Checks',
		description: 'Evaluates conditions against variables and sets the bot status (Success, Fail, Ban, Retry, Custom). Multiple keychains are checked in order.',
		parameters: [
			{ name: 'keychains', type: 'array', required: true, description: 'List of keychains, each with a result status and conditions' },
		],
		codeExample: `Keychain 1: SUCCESS when data.RESPONSECODE EqualTo "200"
Keychain 2: BAN when data.RESPONSECODE EqualTo "403"
Keychain 3: FAIL (default)`,
		tips: [
			'Keychains are evaluated top-to-bottom; first match wins',
			'Use Contains for partial string matching, EqualTo for exact match',
			'Exists/NotExists checks if a variable is set (non-empty)',
			'GreaterThan/LessThan compare numeric values',
		],
		relatedBlocks: ['HttpRequest', 'IfElse', 'CaseSwitch'],
		rustCode: `for keychain in &settings.keychains {
    let all_match = keychain.conditions.iter().all(|cond| {
        let left = self.variables.get(&cond.source);
        match cond.comparison {
            EqualTo => left == cond.value,
            Contains => left.contains(&cond.value),
            GreaterThan => left.parse::<f64>() > cond.value.parse::<f64>(),
            LessThan => left.parse::<f64>() < cond.value.parse::<f64>(),
            Exists => !left.is_empty(),
            NotExists => left.is_empty(),
            MatchesRegex => Regex::new(&cond.value).map(|r| r.is_match(&left)).unwrap_or(false),
            // ... more comparisons
        }
    });
    if all_match {
        self.bot_status = keychain.result.clone(); // SUCCESS, FAIL, BAN, etc.
        break;
    }
}`,
	},
];
