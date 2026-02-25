import type { BlockDoc } from './types';

export const DATA_DOCS: BlockDoc[] = [
	{
		type: 'FileSystem',
		name: 'File System',
		category: 'FileSystem',
		description: 'Performs file and folder operations: read, write, append, copy, move, delete, exists checks, and directory listing.',
		parameters: [
			{ name: 'op', type: 'enum', required: true, description: 'File system operation to perform' },
			{ name: 'path', type: 'string', required: true, description: 'File or folder path. Supports <VAR> interpolation.' },
			{ name: 'dest_path', type: 'string', required: false, description: 'Destination path for Copy and Move operations' },
			{ name: 'content', type: 'string', required: false, description: 'Content to write or append. Supports <VAR> interpolation.' },
			{ name: 'encoding', type: 'string', required: false, description: 'Text encoding for read/write operations', default: 'utf8' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'RESULT' },
			{ name: 'capture', type: 'boolean', required: false, description: 'Mark result as captured output', default: 'false' },
		],
		codeExample: `Op: FileRead
Path: C:\\data\\tokens.txt
→ RESULT = full file contents as a string

Op: FileExists
Path: <FILE_PATH>
→ RESULT = "true" or "false"

Op: GetFilesInFolder
Path: C:\\data\\
→ RESULT = ["C:\\data\\a.txt","C:\\data\\b.txt"] (JSON array)`,
		tips: [
			'FileReadBytes stores bytes as comma-separated decimal integers',
			'FileWriteBytes expects comma-separated decimal integers in content',
			'GetFilesInFolder returns a JSON array of full file paths (files only, not subdirs)',
			'FileReadLines returns a JSON array of strings, one per line',
			'Use safe mode on this block if the path may not exist',
		],
		relatedBlocks: ['SetVariable', 'Log', 'ConversionFunction'],
		rustCode: `let path = self.variables.interpolate(&settings.path);
let result = match settings.op {
    FileRead => std::fs::read_to_string(&path)?,
    FileReadLines => {
        let text = std::fs::read_to_string(&path)?;
        serde_json::to_string(&text.lines().collect::<Vec<_>>())?
    }
    FileWrite => { std::fs::write(&path, content.as_bytes())?; String::new() }
    FileExists => std::path::Path::new(&path).is_file().to_string(),
    FolderExists => std::path::Path::new(&path).is_dir().to_string(),
    GetFilesInFolder => {
        let files: Vec<String> = std::fs::read_dir(&path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter_map(|e| e.path().to_str().map(|s| s.to_string()))
            .collect();
        serde_json::to_string(&files)?
    }
    // ...
};
self.variables.set_user(&settings.output_var, result, settings.capture);`,
	},
];
