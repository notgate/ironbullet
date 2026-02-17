// Temporary debug test module
#[cfg(test)]
mod tests {
    use crate::pipeline::Pipeline;
    use crate::pipeline::block::*;
    use crate::export::rust_codegen;

    /// Test: Simulate adding blocks one by one and generating code each time
    /// This mimics exactly what the frontend does when user adds blocks
    #[test]
    fn test_incremental_block_addition_codegen() {
        let mut pipeline = Pipeline::default();
        println!("=== Incremental block addition test ===\n");

        // Empty pipeline
        let code = rust_codegen::generate_rust_code(&pipeline);
        println!("0 blocks: {} chars", code.len());
        assert!(code.contains("async fn main"), "Should have main function");

        // Add HTTP Request
        pipeline.blocks.push(Block::new(BlockType::HttpRequest));
        let code = rust_codegen::generate_rust_code(&pipeline);
        println!("1 block (HTTP): {} chars", code.len());
        assert!(code.contains("client.get"), "Should have HTTP request code");

        // Add ParseLR
        pipeline.blocks.push(Block::new(BlockType::ParseLR));
        let code = rust_codegen::generate_rust_code(&pipeline);
        println!("2 blocks (+ParseLR): {} chars", code.len());
        assert!(code.contains("find(left)"), "Should have ParseLR code");

        // Add TcpRequest
        pipeline.blocks.push(Block::new(BlockType::TcpRequest));
        let code = rust_codegen::generate_rust_code(&pipeline);
        println!("3 blocks (+TCP): {} chars", code.len());
        assert!(code.contains("TcpStream"), "Should have TCP code");

        // Add CaptchaSolver
        pipeline.blocks.push(Block::new(BlockType::CaptchaSolver));
        let code = rust_codegen::generate_rust_code(&pipeline);
        println!("4 blocks (+Captcha): {} chars", code.len());
        assert!(code.contains("capsolver") || code.contains("createTask"), "Should have captcha code");

        // Add CloudflareBypass
        pipeline.blocks.push(Block::new(BlockType::CloudflareBypass));
        let code = rust_codegen::generate_rust_code(&pipeline);
        println!("5 blocks (+CF Bypass): {} chars", code.len());
        assert!(code.contains("FlareSolverr") || code.contains("flaresolverr"), "Should have CF bypass code");

        // Add LaravelCsrf
        pipeline.blocks.push(Block::new(BlockType::LaravelCsrf));
        let code = rust_codegen::generate_rust_code(&pipeline);
        println!("6 blocks (+Laravel): {} chars", code.len());
        assert!(code.contains("csrf") || code.contains("CSRF"), "Should have CSRF code");

        // Add KeyCheck
        pipeline.blocks.push(Block::new(BlockType::KeyCheck));
        let code = rust_codegen::generate_rust_code(&pipeline);
        println!("7 blocks (+KeyCheck): {} chars", code.len());

        println!("\n=== Final generated code ===");
        println!("{}", code);
    }

    /// Test: Simulate frontend JSON exactly as it would arrive via IPC
    /// with the _tab_id field and verify full roundtrip
    #[test]
    fn test_full_ipc_roundtrip_simulation() {
        println!("=== Full IPC roundtrip simulation ===\n");

        // Step 1: Create a pipeline from Rust (like add_block does)
        let mut pipeline = Pipeline::default();
        pipeline.blocks.push(Block::new(BlockType::HttpRequest));
        pipeline.blocks.push(Block::new(BlockType::FtpRequest));
        pipeline.blocks.push(Block::new(BlockType::CaptchaSolver));
        pipeline.blocks.push(Block::new(BlockType::CloudflareBypass));

        // Step 2: Serialize the pipeline (like frontend would receive from pipeline_loaded)
        let pipeline_json = serde_json::to_value(&pipeline).unwrap();
        println!("Serialized pipeline: {} blocks", pipeline.blocks.len());

        // Step 3: Wrap in IPC data (like frontend's send() function does)
        let ipc_data = serde_json::json!({
            "pipeline": pipeline_json,
            "_tab_id": "test-tab-123"
        });

        // Step 4: Extract and deserialize (like the generate_code handler does)
        let p = ipc_data.get("pipeline").unwrap();
        let deserialized = serde_json::from_value::<Pipeline>(p.clone());
        match &deserialized {
            Ok(p) => {
                println!("Deserialized OK: {} blocks", p.blocks.len());
                for (i, b) in p.blocks.iter().enumerate() {
                    println!("  block[{}]: {:?} '{}'", i, b.block_type, b.label);
                }
            }
            Err(e) => {
                panic!("Deserialization FAILED: {}", e);
            }
        }

        // Step 5: Generate code
        let code = rust_codegen::generate_rust_code(&deserialized.unwrap());
        println!("\nGenerated {} chars of code", code.len());
        println!("{}", code);

        // Verify all block types are represented
        assert!(code.contains("client.get"), "HTTP Request should generate wreq code");
        assert!(code.contains("TcpStream::connect") || code.contains("FTP"), "FTP should generate code");
        assert!(code.contains("capsolver") || code.contains("createTask"), "Captcha should generate code");
        assert!(code.contains("FlareSolverr") || code.contains("flaresolverr"), "CF should generate code");
    }

    /// Test: Verify Block serialization format matches frontend expectations
    #[test]
    fn test_block_serialization_matches_frontend() {
        println!("=== Block serialization format check ===\n");

        // Create blocks from Rust side (like add_block handler does)
        let test_blocks = vec![
            BlockType::HttpRequest,
            BlockType::TcpRequest,
            BlockType::UdpRequest,
            BlockType::FtpRequest,
            BlockType::SshRequest,
            BlockType::ImapRequest,
            BlockType::SmtpRequest,
            BlockType::PopRequest,
            BlockType::CaptchaSolver,
            BlockType::CloudflareBypass,
            BlockType::LaravelCsrf,
            BlockType::KeyCheck,
            BlockType::ParseLR,
            BlockType::ParseRegex,
            BlockType::ParseJSON,
            BlockType::ParseCSS,
            BlockType::ParseXPath,
            BlockType::StringFunction,
            BlockType::CryptoFunction,
            BlockType::DateFunction,
            BlockType::RandomData,
            BlockType::Group,
        ];

        for bt in test_blocks {
            let block = Block::new(bt);
            let json = serde_json::to_value(&block).unwrap();

            // Check that the settings have a "type" field (serde tag)
            let settings = json.get("settings").unwrap();
            let type_tag = settings.get("type").and_then(|v| v.as_str());
            println!("{:?}: type tag = {:?}", bt, type_tag);
            assert!(type_tag.is_some(), "{:?} settings missing 'type' tag", bt);

            // The type tag should match the enum variant name
            let expected_tag = format!("{:?}", bt);
            assert_eq!(type_tag.unwrap(), expected_tag,
                "{:?} type tag mismatch: got '{}', expected '{}'",
                bt, type_tag.unwrap(), expected_tag);

            // Verify the block roundtrips through JSON
            let json_str = serde_json::to_string(&block).unwrap();
            let back: Result<Block, _> = serde_json::from_str(&json_str);
            assert!(back.is_ok(), "{:?} failed roundtrip: {}", bt, back.unwrap_err());
        }
    }

    /// Test: Simulate pipeline with frontend-style UUIDs (crypto.randomUUID format)
    #[test]
    fn test_pipeline_with_crypto_random_uuid() {
        // crypto.randomUUID() produces lowercase UUIDs like:
        // "3b241101-e2bb-4d7a-8613-e4d4e2e57e1c"
        let json = r#"{
            "id": "3b241101-e2bb-4d7a-8613-e4d4e2e57e1c",
            "name": "Test Config",
            "author": "",
            "created": "2026-02-16T00:00:00.000Z",
            "modified": "2026-02-16T00:00:00.000Z",
            "blocks": [{
                "id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
                "block_type": "HttpRequest",
                "label": "HTTP Request",
                "disabled": false,
                "safe_mode": false,
                "settings": {
                    "type": "HttpRequest",
                    "method": "GET",
                    "url": "https://example.com",
                    "headers": [["User-Agent", "Mozilla/5.0"]],
                    "body": "",
                    "body_type": "None",
                    "content_type": "",
                    "follow_redirects": true,
                    "max_redirects": 5,
                    "timeout_ms": 10000,
                    "auto_redirect": true,
                    "basic_auth": null,
                    "http_version": "2",
                    "response_var": "SOURCE",
                    "custom_cookies": ""
                }
            },{
                "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "block_type": "CloudflareBypass",
                "label": "Cloudflare Bypass",
                "disabled": false,
                "safe_mode": false,
                "settings": {
                    "type": "CloudflareBypass",
                    "url": "https://target.com",
                    "flaresolverr_url": "http://localhost:8191/v1",
                    "max_timeout_ms": 60000,
                    "output_var": "CF_COOKIES",
                    "capture": false
                }
            }],
            "startup_blocks": [],
            "data_settings": {"wordlist_type": "Credentials", "separator": ":", "slices": ["USER","PASS"]},
            "proxy_settings": {"proxy_mode": "None", "proxy_sources": [], "ban_duration_secs": 300, "max_retries_before_ban": 3, "cpm_per_proxy": 0, "proxy_groups": [], "active_group": ""},
            "browser_settings": {"browser": "chrome", "ja3": null, "http2_fingerprint": null, "user_agent": null},
            "runner_settings": {"threads": 100, "skip": 0, "take": 0, "continue_statuses": ["Retry"], "custom_status_name": "CUSTOM", "max_retries": 3, "concurrent_per_proxy": 0, "start_threads_gradually": true, "gradual_delay_ms": 100, "automatic_thread_count": false, "lower_threads_on_retry": false, "retry_thread_reduction_pct": 25, "pause_on_ratelimit": false, "only_proxyless": false},
            "output_settings": {"save_to_file": true, "save_to_database": false, "include_response": false, "output_directory": "results", "output_format": "{data} | {captures}", "database_path": "", "output_format_type": "Txt", "capture_filters": []}
        }"#;

        let result: Result<Pipeline, _> = serde_json::from_str(json);
        match &result {
            Ok(p) => {
                println!("Pipeline with crypto.randomUUID format: {} blocks", p.blocks.len());
                let code = rust_codegen::generate_rust_code(p);
                println!("Generated {} chars:\n{}", code.len(), code);
                assert!(code.contains("client.get"), "HTTP should produce code");
                assert!(code.contains("flaresolverr") || code.contains("FlareSolverr"), "CF bypass should produce code");
            }
            Err(e) => panic!("FAIL: {}", e),
        }
    }

    /// Test: What happens with an empty blocks pipeline (this is the default state)
    #[test]
    fn test_empty_pipeline_generates_boilerplate() {
        let pipeline = Pipeline::default();
        let code = rust_codegen::generate_rust_code(&pipeline);
        println!("Empty pipeline code ({} chars):\n{}", code.len(), code);
        assert!(code.contains("async fn main"), "Should have main function");
        assert!(code.contains("Ok(())"), "Should have Ok(())");
        assert!(code.len() > 100, "Should have at least boilerplate code");
    }
}
