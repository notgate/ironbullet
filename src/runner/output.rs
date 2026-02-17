use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::pipeline::{BotStatus, CaptureFilter, CaptureFilterType, OutputFormat, OutputSettings};
use super::HitResult;

/// Manages output file writing in TXT/CSV/JSON formats
pub struct OutputWriter {
    settings: OutputSettings,
    config_name: String,
    dir: PathBuf,
    /// Per-status file writers (lazily created)
    writers: Mutex<HashMap<String, StatusWriter>>,
}

struct StatusWriter {
    writer: BufWriter<File>,
    format: OutputFormat,
    has_header: bool,
    json_count: usize,
}

impl OutputWriter {
    pub fn new(settings: &OutputSettings, config_name: &str) -> Self {
        let dir = PathBuf::from(&settings.output_directory);
        let _ = fs::create_dir_all(&dir);
        Self {
            settings: settings.clone(),
            config_name: sanitize_filename(config_name),
            dir,
            writers: Mutex::new(HashMap::new()),
        }
    }

    /// Write a hit result to the appropriate output file
    pub fn write_hit(&self, hit: &HitResult, status: BotStatus) {
        if !self.settings.save_to_file {
            return;
        }

        // Apply capture filters
        let captures = apply_capture_filters(&hit.captures, &self.settings.capture_filters);

        let status_str = format!("{:?}", status);
        let mut writers = self.writers.lock().unwrap();

        let writer = writers.entry(status_str.clone()).or_insert_with(|| {
            let ext = match self.settings.output_format_type {
                OutputFormat::Txt => "txt",
                OutputFormat::Csv => "csv",
                OutputFormat::Json => "json",
            };
            let filename = format!("{}_{}.{}", self.config_name, status_str, ext);
            let path = self.dir.join(&filename);
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .unwrap_or_else(|_| File::create(&path).unwrap());
            let mut sw = StatusWriter {
                writer: BufWriter::new(file),
                format: self.settings.output_format_type.clone(),
                has_header: false,
                json_count: 0,
            };
            // JSON: start array
            if matches!(sw.format, OutputFormat::Json) {
                let _ = sw.writer.write_all(b"[\n");
            }
            sw
        });

        match writer.format {
            OutputFormat::Txt => {
                let line = format_hit_line(&self.settings.output_format, &hit.data_line, &captures);
                let _ = writeln!(writer.writer, "{}", line);
            }
            OutputFormat::Csv => {
                // Write header on first hit
                if !writer.has_header {
                    let mut header = String::from("data");
                    let mut keys: Vec<&String> = captures.keys().collect();
                    keys.sort();
                    for key in &keys {
                        header.push(',');
                        header.push_str(&csv_escape(key));
                    }
                    if hit.proxy.is_some() {
                        header.push_str(",proxy");
                    }
                    let _ = writeln!(writer.writer, "{}", header);
                    writer.has_header = true;
                }
                let mut line = csv_escape(&hit.data_line);
                let mut keys: Vec<&String> = captures.keys().collect();
                keys.sort();
                for key in &keys {
                    line.push(',');
                    line.push_str(&csv_escape(captures.get(*key).unwrap_or(&String::new())));
                }
                if let Some(ref proxy) = hit.proxy {
                    line.push(',');
                    line.push_str(&csv_escape(proxy));
                }
                let _ = writeln!(writer.writer, "{}", line);
            }
            OutputFormat::Json => {
                if writer.json_count > 0 {
                    let _ = writer.writer.write_all(b",\n");
                }
                let entry = serde_json::json!({
                    "data": hit.data_line,
                    "captures": captures,
                    "proxy": hit.proxy,
                });
                let _ = writer.writer.write_all(
                    serde_json::to_string_pretty(&entry).unwrap_or_default().as_bytes()
                );
                writer.json_count += 1;
            }
        }
        let _ = writer.writer.flush();
    }

    /// Finalize all output files (close JSON arrays, etc.)
    pub fn flush(&self) {
        let mut writers = self.writers.lock().unwrap();
        for (_, sw) in writers.iter_mut() {
            if matches!(sw.format, OutputFormat::Json) {
                let _ = sw.writer.write_all(b"\n]\n");
            }
            let _ = sw.writer.flush();
        }
    }
}

/// Apply capture filters to a captures map, returning filtered captures
pub fn apply_capture_filters(
    captures: &HashMap<String, String>,
    filters: &[CaptureFilter],
) -> HashMap<String, String> {
    if filters.is_empty() {
        return captures.clone();
    }

    captures.iter()
        .filter(|(key, value)| {
            filters.iter().all(|f| {
                // Check if this filter applies to this variable
                let applies = f.variable_name == "*" || f.variable_name == **key;
                if !applies {
                    return true; // Filter doesn't apply to this var, keep it
                }
                let matches = match f.filter_type {
                    CaptureFilterType::Contains => value.contains(&f.value),
                    CaptureFilterType::Equals => **value == f.value,
                    CaptureFilterType::StartsWith => value.starts_with(&f.value),
                    CaptureFilterType::EndsWith => value.ends_with(&f.value),
                    CaptureFilterType::MatchesRegex => {
                        regex::Regex::new(&f.value)
                            .map(|re| re.is_match(value))
                            .unwrap_or(false)
                    }
                    CaptureFilterType::MinLength => {
                        f.value.parse::<usize>().map(|min| value.len() >= min).unwrap_or(true)
                    }
                    CaptureFilterType::MaxLength => {
                        f.value.parse::<usize>().map(|max| value.len() <= max).unwrap_or(true)
                    }
                    CaptureFilterType::NotEmpty => !value.is_empty(),
                };
                if f.negate { !matches } else { matches }
            })
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn format_hit_line(template: &str, data_line: &str, captures: &HashMap<String, String>) -> String {
    let captures_str = captures.iter()
        .map(|(k, v)| format!("{} = {}", k, v))
        .collect::<Vec<_>>()
        .join(" | ");

    template
        .replace("{data}", data_line)
        .replace("{captures}", &captures_str)
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
