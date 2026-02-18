import type { PluginBlock } from './types';

export function sanitize(s: string): string {
	return s.replace(/"/g, '\\"');
}

export function toSnake(s: string): string {
	return s.replace(/[A-Z]/g, (c, i) => (i ? '_' : '') + c.toLowerCase()).replace(/\s+/g, '_');
}

export function generateSchemaJson(b: PluginBlock): string {
	const props: Record<string, any> = {};
	for (const f of b.settingsFields) {
		props[f.name] = { type: f.type === 'number' ? 'number' : f.type === 'boolean' ? 'boolean' : 'string', title: f.name.replace(/_/g, ' '), default: f.default };
	}
	return JSON.stringify({ type: 'object', properties: props });
}

export function generateDefaultsJson(b: PluginBlock): string {
	const defaults: Record<string, any> = {};
	for (const f of b.settingsFields) {
		if (f.type === 'number') defaults[f.name] = parseFloat(f.default) || 0;
		else if (f.type === 'boolean') defaults[f.name] = f.default === 'true';
		else defaults[f.name] = f.default;
	}
	return JSON.stringify(defaults);
}

export function generateCargoToml(
	pluginName: string,
	pluginVersion: string,
	extraDeps: Array<{ name: string; version: string; features: string }>,
): string {
	let toml = `[package]\nname = "${sanitize(pluginName).toLowerCase().replace(/\s+/g, '-')}"\nversion = "${pluginVersion}"\nedition = "2021"\n\n[lib]\ncrate-type = ["cdylib"]\n\n[dependencies]\nserde = { version = "1", features = ["derive"] }\nserde_json = "1"\n`;
	for (const dep of extraDeps) {
		if (!dep.name) continue;
		if (dep.features) {
			toml += `${dep.name} = { version = "${dep.version || '*'}", features = [${dep.features.split(',').map(f => `"${f.trim()}"`).join(', ')}] }\n`;
		} else {
			toml += `${dep.name} = "${dep.version || '*'}"\n`;
		}
	}
	return toml;
}

export function generateLibRs(
	pluginName: string,
	pluginVersion: string,
	pluginAuthor: string,
	pluginDescription: string,
	blocks: PluginBlock[],
): string {
	const blockCount = blocks.length;
	let code = `use std::collections::HashMap;\nuse std::ffi::{c_char, CStr, CString};\nuse std::sync::OnceLock;\n\n`;

	code += `// ── ABI structs (must match ironbullet's plugin::abi) ──\n\n`;
	code += `#[repr(C)]\npub struct PluginInfo {\n    pub name: *const c_char,\n    pub version: *const c_char,\n    pub author: *const c_char,\n    pub description: *const c_char,\n    pub block_count: u32,\n}\n\n`;
	code += `unsafe impl Send for PluginInfo {}\nunsafe impl Sync for PluginInfo {}\n\n`;
	code += `#[repr(C)]\npub struct BlockInfo {\n    pub block_type_name: *const c_char,\n    pub label: *const c_char,\n    pub category: *const c_char,\n    pub color: *const c_char,\n    pub icon: *const c_char,\n    pub settings_schema_json: *const c_char,\n    pub default_settings_json: *const c_char,\n}\n\n`;
	code += `unsafe impl Send for BlockInfo {}\nunsafe impl Sync for BlockInfo {}\n\n`;
	code += `#[repr(C)]\npub struct ExecuteResult {\n    pub success: bool,\n    pub updated_variables_json: *const c_char,\n    pub log_message: *const c_char,\n    pub error_message: *const c_char,\n}\n\n`;

	code += `// ── Helpers ──\n\n`;
	code += `fn leak_cstring(s: &str) -> *const c_char {\n    CString::new(s).unwrap().into_raw() as *const c_char\n}\n\n`;
	code += `fn alloc_cstring(s: &str) -> *const c_char {\n    CString::new(s).unwrap().into_raw() as *const c_char\n}\n\n`;

	code += `// ── Plugin info ──\n\n`;
	code += `static PLUGIN_INFO: OnceLock<PluginInfo> = OnceLock::new();\n`;
	for (let i = 0; i < blockCount; i++) {
		code += `static BLOCK_INFO_${i}: OnceLock<BlockInfo> = OnceLock::new();\n`;
	}

	code += `\nfn get_plugin_info() -> &'static PluginInfo {\n`;
	code += `    PLUGIN_INFO.get_or_init(|| PluginInfo {\n`;
	code += `        name: leak_cstring("${sanitize(pluginName)}"),\n`;
	code += `        version: leak_cstring("${sanitize(pluginVersion)}"),\n`;
	code += `        author: leak_cstring("${sanitize(pluginAuthor)}"),\n`;
	code += `        description: leak_cstring("${sanitize(pluginDescription)}"),\n`;
	code += `        block_count: ${blockCount},\n`;
	code += `    })\n}\n\n`;

	for (let i = 0; i < blockCount; i++) {
		const b = blocks[i];
		const typeName = `${pluginName}.${b.name}`;
		const schema = generateSchemaJson(b);
		const defaults = generateDefaultsJson(b);

		code += `fn get_block_info_${i}() -> &'static BlockInfo {\n`;
		code += `    BLOCK_INFO_${i}.get_or_init(|| BlockInfo {\n`;
		code += `        block_type_name: leak_cstring("${sanitize(typeName)}"),\n`;
		code += `        label: leak_cstring("${sanitize(b.label)}"),\n`;
		code += `        category: leak_cstring("${sanitize(b.category)}"),\n`;
		code += `        color: leak_cstring("${sanitize(b.color)}"),\n`;
		code += `        icon: leak_cstring("puzzle"),\n`;
		code += `        settings_schema_json: leak_cstring(r#"${schema}"#),\n`;
		code += `        default_settings_json: leak_cstring(r#"${defaults}"#),\n`;
		code += `    })\n}\n\n`;
	}

	code += `// ── Exports ──\n\n`;
	code += `#[no_mangle]\npub extern "C" fn plugin_info() -> *const PluginInfo {\n    get_plugin_info() as *const PluginInfo\n}\n\n`;

	code += `#[no_mangle]\npub extern "C" fn plugin_block_info(index: u32) -> *const BlockInfo {\n`;
	code += `    match index {\n`;
	for (let i = 0; i < blockCount; i++) {
		code += `        ${i} => get_block_info_${i}() as *const BlockInfo,\n`;
	}
	code += `        _ => std::ptr::null(),\n`;
	code += `    }\n}\n\n`;

	code += `#[no_mangle]\npub extern "C" fn plugin_execute(\n    block_index: u32,\n    settings_json: *const c_char,\n    variables_json: *const c_char,\n) -> *const ExecuteResult {\n`;
	code += `    let settings_str = if settings_json.is_null() {\n        "{}".to_string()\n    } else {\n        unsafe { CStr::from_ptr(settings_json).to_string_lossy().to_string() }\n    };\n\n`;
	code += `    let vars_str = if variables_json.is_null() {\n        "{}".to_string()\n    } else {\n        unsafe { CStr::from_ptr(variables_json).to_string_lossy().to_string() }\n    };\n\n`;
	code += `    let settings: HashMap<String, serde_json::Value> =\n        serde_json::from_str(&settings_str).unwrap_or_default();\n`;
	code += `    let mut vars: HashMap<String, String> =\n        serde_json::from_str(&vars_str).unwrap_or_default();\n\n`;

	code += `    match block_index {\n`;
	for (let i = 0; i < blockCount; i++) {
		const b = blocks[i];
		code += `        ${i} => {\n`;
		code += `            // ── ${b.name}: ${b.label} ──\n`;
		const fieldNames: string[] = [];
		for (const f of b.settingsFields) {
			const vn = toSnake(f.name);
			fieldNames.push(vn);
			if (f.type === 'string') {
				code += `            let ${vn} = settings.get("${f.name}")\n                .and_then(|v| v.as_str())\n                .unwrap_or("${sanitize(f.default)}")\n                .to_string();\n`;
			} else if (f.type === 'number') {
				code += `            let ${vn} = settings.get("${f.name}")\n                .and_then(|v| v.as_f64())\n                .unwrap_or(${f.default || '0'}.0);\n`;
			} else if (f.type === 'boolean') {
				code += `            let ${vn} = settings.get("${f.name}")\n                .and_then(|v| v.as_bool())\n                .unwrap_or(${f.default || 'false'});\n`;
			}
		}
		code += `\n            // TODO: Implement your block logic here\n`;
		if (fieldNames.length > 0) {
			code += `            let result_value = format!("${b.name}(${fieldNames.map(() => '{}').join(', ')})", ${fieldNames.join(', ')});\n\n`;
		} else {
			code += `            let result_value = "${b.name}".to_string();\n\n`;
		}
		code += `            vars.insert("PLUGIN_RESULT".to_string(), result_value.clone());\n`;
		code += `            let log_msg = format!("${b.name}: produced '{}'", result_value);\n`;
		code += `            make_result(true, &vars, &log_msg, "")\n`;
		code += `        }\n`;
	}
	code += `        _ => make_result(false, &vars, "", "Unknown block index"),\n`;
	code += `    }\n}\n\n`;

	code += `fn make_result(\n    success: bool,\n    vars: &HashMap<String, String>,\n    log: &str,\n    error: &str,\n) -> *const ExecuteResult {\n`;
	code += `    let vars_json = serde_json::to_string(vars).unwrap_or_else(|_| "{}".to_string());\n`;
	code += `    let result = Box::new(ExecuteResult {\n`;
	code += `        success,\n`;
	code += `        updated_variables_json: alloc_cstring(&vars_json),\n`;
	code += `        log_message: alloc_cstring(log),\n`;
	code += `        error_message: if error.is_empty() { std::ptr::null() } else { alloc_cstring(error) },\n`;
	code += `    });\n`;
	code += `    Box::into_raw(result) as *const ExecuteResult\n}\n\n`;

	code += `#[no_mangle]\npub extern "C" fn plugin_free_string(ptr: *const c_char) {\n    if !ptr.is_null() {\n        unsafe {\n            drop(CString::from_raw(ptr as *mut c_char));\n        }\n    }\n}\n`;

	return code;
}
