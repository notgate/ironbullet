use super::*;

fn parse_bytes(s: &str) -> Vec<u8> {
    s.split(',')
        .filter_map(|p| p.trim().parse::<u8>().ok())
        .collect()
}

fn bytes_to_csv(bytes: &[u8]) -> String {
    bytes.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(",")
}

fn number_to_words(n: i64) -> String {
    if n < 0 {
        return format!("negative {}", number_to_words(-n));
    }
    if n == 0 {
        return "zero".into();
    }
    let ones = ["", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
                "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen",
                "seventeen", "eighteen", "nineteen"];
    let tens = ["", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"];

    fn below_1000(n: i64, ones: &[&str], tens: &[&str]) -> String {
        if n == 0 { return String::new(); }
        if n < 20 { return ones[n as usize].into(); }
        if n < 100 {
            let t = tens[(n / 10) as usize];
            let o = ones[(n % 10) as usize];
            return if o.is_empty() { t.into() } else { format!("{}-{}", t, o) };
        }
        let h = ones[(n / 100) as usize];
        let rest = n % 100;
        if rest == 0 {
            format!("{} hundred", h)
        } else {
            format!("{} hundred {}", h, below_1000(rest, ones, tens))
        }
    }

    let mut parts: Vec<String> = Vec::new();
    let scales = [(1_000_000_000_000i64, "trillion"), (1_000_000_000, "billion"),
                  (1_000_000, "million"), (1_000, "thousand")];
    let mut remaining = n;
    for (scale, label) in &scales {
        if remaining >= *scale {
            parts.push(format!("{} {}", below_1000(remaining / scale, &ones, &tens), label));
            remaining %= scale;
        }
    }
    if remaining > 0 {
        parts.push(below_1000(remaining, &ones, &tens));
    }
    parts.join(" ")
}

fn words_to_number(s: &str) -> i64 {
    let s = s.trim().to_lowercase();
    let is_negative = s.starts_with("negative ");
    let s = s.trim_start_matches("negative ").trim();

    let word_map: &[(&str, i64)] = &[
        ("zero",0),("one",1),("two",2),("three",3),("four",4),("five",5),
        ("six",6),("seven",7),("eight",8),("nine",9),("ten",10),
        ("eleven",11),("twelve",12),("thirteen",13),("fourteen",14),("fifteen",15),
        ("sixteen",16),("seventeen",17),("eighteen",18),("nineteen",19),
        ("twenty",20),("thirty",30),("forty",40),("fifty",50),
        ("sixty",60),("seventy",70),("eighty",80),("ninety",90),
        ("hundred",100),("thousand",1_000),("million",1_000_000),
        ("billion",1_000_000_000),("trillion",1_000_000_000_000),
    ];

    let mut total: i64 = 0;
    let mut current: i64 = 0;

    for token in s.split(|c: char| c == ' ' || c == '-') {
        let token = token.trim();
        if token.is_empty() { continue; }
        if let Some(&(_, val)) = word_map.iter().find(|(w, _)| *w == token) {
            if val == 100 {
                current *= 100;
            } else if val >= 1_000 {
                total += (current + if current == 0 { 1 } else { 0 }) * val;
                current = 0;
            } else {
                current += val;
            }
        }
    }
    total += current;
    if is_negative { -total } else { total }
}

fn readable_size(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    if bytes < 0 { return format!("{} B", bytes); }
    let mut val = bytes as f64;
    let mut unit_idx = 0;
    while val >= 1024.0 && unit_idx < UNITS.len() - 1 {
        val /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.2} {}", val, UNITS[unit_idx])
    }
}

impl ExecutionContext {
    pub(super) fn execute_data_conversion(&mut self, settings: &DataConversionSettings) -> crate::error::Result<()> {
        let input = self.variables.resolve_input(&settings.input_var);

        let result = match settings.op {
            DataConversionOp::HexToBytes => {
                let hex = input.chars().filter(|c| c.is_ascii_hexdigit()).collect::<String>();
                let bytes: Vec<u8> = (0..hex.len())
                    .step_by(2)
                    .filter_map(|i| u8::from_str_radix(&hex[i..i+2], 16).ok())
                    .collect();
                bytes_to_csv(&bytes)
            }
            DataConversionOp::BytesToHex => {
                let bytes = parse_bytes(&input);
                bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
            }
            DataConversionOp::Base64ToBytes => {
                use base64::Engine;
                match base64::engine::general_purpose::STANDARD.decode(input.trim()) {
                    Ok(bytes) => bytes_to_csv(&bytes),
                    Err(_) => String::new(),
                }
            }
            DataConversionOp::BytesToBase64 => {
                use base64::Engine;
                let bytes = parse_bytes(&input);
                base64::engine::general_purpose::STANDARD.encode(&bytes)
            }
            DataConversionOp::Base64ToString => {
                use base64::Engine;
                match base64::engine::general_purpose::STANDARD.decode(input.trim()) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                    Err(_) => String::new(),
                }
            }
            DataConversionOp::StringToBytes => {
                let bytes: Vec<u8> = match settings.encoding.as_str() {
                    "utf16" | "utf-16" => {
                        let utf16: Vec<u16> = input.encode_utf16().collect();
                        utf16.iter().flat_map(|u| u.to_be_bytes()).collect()
                    }
                    "ascii" => input.bytes().map(|b| b & 0x7F).collect(),
                    _ => input.into_bytes(),
                };
                bytes_to_csv(&bytes)
            }
            DataConversionOp::BytesToString => {
                let bytes = parse_bytes(&input);
                match settings.encoding.as_str() {
                    "utf16" | "utf-16" => {
                        let u16s: Vec<u16> = bytes.chunks(2)
                            .filter_map(|c| if c.len() == 2 { Some(u16::from_be_bytes([c[0], c[1]])) } else { None })
                            .collect();
                        String::from_utf16_lossy(&u16s)
                    }
                    _ => String::from_utf8_lossy(&bytes).to_string(),
                }
            }
            DataConversionOp::BigIntToBytes => {
                use num_bigint::BigInt;
                use std::str::FromStr;
                match BigInt::from_str(input.trim()) {
                    Ok(n) => {
                        let (_, bytes) = n.to_bytes_be();
                        bytes_to_csv(&bytes)
                    }
                    Err(_) => String::new(),
                }
            }
            DataConversionOp::BytesToBigInt => {
                use num_bigint::BigUint;
                let bytes = parse_bytes(&input);
                BigUint::from_bytes_be(&bytes).to_string()
            }
            DataConversionOp::BinaryStringToBytes => {
                let cleaned: String = input.chars().filter(|c| *c == '0' || *c == '1').collect();
                let bytes: Vec<u8> = cleaned.as_bytes().chunks(8)
                    .filter_map(|chunk| {
                        let s = std::str::from_utf8(chunk).ok()?;
                        u8::from_str_radix(s, 2).ok()
                    })
                    .collect();
                bytes_to_csv(&bytes)
            }
            DataConversionOp::BytesToBinaryString => {
                let bytes = parse_bytes(&input);
                bytes.iter().map(|b| format!("{:08b}", b)).collect::<Vec<_>>().join(" ")
            }
            DataConversionOp::ReadableSize => {
                let n: i64 = input.trim().parse().unwrap_or(0);
                readable_size(n)
            }
            DataConversionOp::IntToBytes => {
                let n: i64 = input.trim().parse().unwrap_or(0);
                let count = settings.byte_count.clamp(1, 8) as usize;
                let all_bytes = if settings.endianness == "little" {
                    n.to_le_bytes()
                } else {
                    n.to_be_bytes()
                };
                let bytes = if settings.endianness == "little" {
                    all_bytes[..count].to_vec()
                } else {
                    all_bytes[8 - count..].to_vec()
                };
                bytes_to_csv(&bytes)
            }
            DataConversionOp::NumberToWords => {
                let n: i64 = input.trim().parse().unwrap_or(0);
                number_to_words(n)
            }
            DataConversionOp::WordsToNumber => {
                words_to_number(&input).to_string()
            }
            DataConversionOp::SvgToPng => {
                use resvg::usvg;
                use resvg::tiny_skia;
                use base64::Engine;

                let mut opt = usvg::Options::default();
                opt.fontdb_mut().load_system_fonts();
                match usvg::Tree::from_str(&input, &opt) {
                    Ok(tree) => {
                        let size = tree.size();
                        let width = (size.width().ceil() as u32).max(1);
                        let height = (size.height().ceil() as u32).max(1);
                        if let Some(mut pixmap) = tiny_skia::Pixmap::new(width, height) {
                            resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
                            match pixmap.encode_png() {
                                Ok(png_bytes) => base64::engine::general_purpose::STANDARD.encode(&png_bytes),
                                Err(_) => String::new(),
                            }
                        } else {
                            String::new()
                        }
                    }
                    Err(_) => String::new(),
                }
            }
        };

        self.variables.set_user(&settings.output_var, result, settings.capture);
        Ok(())
    }

    pub(super) fn execute_file_system(&mut self, settings: &FileSystemSettings) -> crate::error::Result<()> {
        let path = self.variables.interpolate(&settings.path);
        let dest = self.variables.interpolate(&settings.dest_path);
        let content = self.variables.interpolate(&settings.content);

        let err = |msg: String| crate::error::AppError::Pipeline(msg);

        let result: String = match settings.op {
            FileSystemOp::CreatePath => {
                std::fs::create_dir_all(&path)
                    .map_err(|e| err(format!("CreatePath: {}", e)))?;
                path.clone()
            }
            FileSystemOp::FileAppend => {
                use std::io::Write;
                let mut f = std::fs::OpenOptions::new().append(true).create(true).open(&path)
                    .map_err(|e| err(format!("FileAppend: {}", e)))?;
                f.write_all(content.as_bytes()).map_err(|e| err(format!("FileAppend write: {}", e)))?;
                String::new()
            }
            FileSystemOp::FileAppendLines => {
                use std::io::Write;
                let mut f = std::fs::OpenOptions::new().append(true).create(true).open(&path)
                    .map_err(|e| err(format!("FileAppendLines: {}", e)))?;
                for line in content.lines() {
                    writeln!(f, "{}", line).map_err(|e| err(format!("FileAppendLines write: {}", e)))?;
                }
                String::new()
            }
            FileSystemOp::FileCopy => {
                std::fs::copy(&path, &dest)
                    .map_err(|e| err(format!("FileCopy: {}", e)))?;
                dest.clone()
            }
            FileSystemOp::FileMove => {
                std::fs::rename(&path, &dest)
                    .map_err(|e| err(format!("FileMove: {}", e)))?;
                dest.clone()
            }
            FileSystemOp::FileDelete => {
                std::fs::remove_file(&path)
                    .map_err(|e| err(format!("FileDelete: {}", e)))?;
                String::new()
            }
            FileSystemOp::FileExists => {
                std::path::Path::new(&path).is_file().to_string()
            }
            FileSystemOp::FileRead => {
                std::fs::read_to_string(&path)
                    .map_err(|e| err(format!("FileRead: {}", e)))?
            }
            FileSystemOp::FileReadBytes => {
                let bytes = std::fs::read(&path)
                    .map_err(|e| err(format!("FileReadBytes: {}", e)))?;
                bytes_to_csv(&bytes)
            }
            FileSystemOp::FileReadLines => {
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| err(format!("FileReadLines: {}", e)))?;
                let lines: Vec<&str> = text.lines().collect();
                serde_json::to_string(&lines).unwrap_or_default()
            }
            FileSystemOp::FileWrite => {
                std::fs::write(&path, content.as_bytes())
                    .map_err(|e| err(format!("FileWrite: {}", e)))?;
                String::new()
            }
            FileSystemOp::FileWriteBytes => {
                let bytes = parse_bytes(&content);
                std::fs::write(&path, &bytes)
                    .map_err(|e| err(format!("FileWriteBytes: {}", e)))?;
                String::new()
            }
            FileSystemOp::FileWriteLines => {
                let joined = content.lines().collect::<Vec<_>>().join("\n");
                std::fs::write(&path, joined.as_bytes())
                    .map_err(|e| err(format!("FileWriteLines: {}", e)))?;
                String::new()
            }
            FileSystemOp::FolderDelete => {
                std::fs::remove_dir_all(&path)
                    .map_err(|e| err(format!("FolderDelete: {}", e)))?;
                String::new()
            }
            FileSystemOp::FolderExists => {
                std::path::Path::new(&path).is_dir().to_string()
            }
            FileSystemOp::GetFilesInFolder => {
                let entries: Vec<String> = std::fs::read_dir(&path)
                    .map_err(|e| err(format!("GetFilesInFolder: {}", e)))?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                    .filter_map(|e| e.path().to_str().map(|s| s.to_string()))
                    .collect();
                serde_json::to_string(&entries).unwrap_or_default()
            }
        };

        self.variables.set_user(&settings.output_var, result, settings.capture);
        Ok(())
    }
}
