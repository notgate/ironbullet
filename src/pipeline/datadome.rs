/// DataDome sensor data generation.
/// Credit: glizzykingdreko/datadome-wasm

pub fn generate_datadome_sensor(
    site_url: &str,
    cookie_datadome: &str,
    user_agent: &str,
    _custom_wasm: Option<&[u8]>,
) -> crate::error::Result<String> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    // Navigator properties
    let platform = "Win32";
    let language = "en-US";
    let hardware_concurrency = [4, 8, 12, 16][rng.gen_range(0..4)];
    let device_memory = [4, 8, 16][rng.gen_range(0..3)];
    let max_touch_points = 0;

    // Screen properties
    let screen_w = 1920;
    let screen_h = 1080;
    let color_depth = 24;
    let pixel_ratio = 1.0f64;

    // Generate realistic mouse movements (bezier-curve-like)
    let move_count: u32 = rng.gen_range(15..40);
    let mut mouse_moves = Vec::new();
    let mut t = timestamp - rng.gen_range(3000..8000) as u128;
    let mut x: f64 = rng.gen_range(200.0..800.0);
    let mut y: f64 = rng.gen_range(200.0..600.0);
    for _ in 0..move_count {
        x += rng.gen_range(-30.0..30.0);
        y += rng.gen_range(-20.0..20.0);
        x = x.clamp(0.0, screen_w as f64);
        y = y.clamp(0.0, screen_h as f64);
        t += rng.gen_range(15..80) as u128;
        mouse_moves.push(format!("{},{},{}", t, x as i32, y as i32));
    }
    let mouse_data = mouse_moves.join(";");

    // Timing signals
    let nav_start = timestamp - rng.gen_range(5000..15000) as u128;
    let dom_content_loaded = nav_start + rng.gen_range(300..1500) as u128;
    let load_event = dom_content_loaded + rng.gen_range(200..2000) as u128;

    // Canvas/WebGL fingerprint (static plausible hashes)
    let canvas_hashes = [
        "a1b2c3d4e5f6a7b8", "f8e7d6c5b4a39281", "1a2b3c4d5e6f7a8b",
        "9f8e7d6c5b4a3210", "abcdef0123456789",
    ];
    let webgl_renderers = [
        "ANGLE (NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0)",
        "ANGLE (NVIDIA GeForce GTX 1660 Ti Direct3D11 vs_5_0 ps_5_0)",
        "ANGLE (AMD Radeon RX 580 Direct3D11 vs_5_0 ps_5_0)",
        "ANGLE (Intel(R) UHD Graphics 630 Direct3D11 vs_5_0 ps_5_0)",
        "ANGLE (NVIDIA GeForce RTX 4070 Direct3D11 vs_5_0 ps_5_0)",
    ];
    let canvas_hash = canvas_hashes[rng.gen_range(0..canvas_hashes.len())];
    let webgl_renderer = webgl_renderers[rng.gen_range(0..webgl_renderers.len())];

    // Key events
    let key_count: u32 = rng.gen_range(2..6);
    let mut key_events = Vec::new();
    let mut kt = timestamp - rng.gen_range(1000..3000) as u128;
    for _ in 0..key_count {
        let keycode: u32 = rng.gen_range(65..90); // A-Z
        kt += rng.gen_range(50..200) as u128;
        key_events.push(format!("{}:{}", kt, keycode));
    }
    let key_data = key_events.join(";");

    // Build sensor payload as JSON
    let sensor = serde_json::json!({
        "jsType": "ch",
        "cType": "re",
        "ddk": cookie_datadome,
        "Referer": site_url,
        "request": format!("/{}", &site_url.split('/').skip(3).collect::<Vec<_>>().join("/")),
        "responsePage": "origin",
        "ddv": "4.27.0",
        "events": [
            {"source":{"x":x as i32,"y":y as i32},"message":{"type":"mousemove"},"date": timestamp},
            {"source":{"x":x as i32,"y":y as i32},"message":{"type":"mousedown"},"date": timestamp + 50},
            {"source":{"x":x as i32,"y":y as i32},"message":{"type":"mouseup"},"date": timestamp + 120},
        ],
        "eventCounters": [
            {"type":"mousemove","count":move_count},
            {"type":"mousedown","count":1},
            {"type":"mouseup","count":1},
            {"type":"touchstart","count":0},
            {"type":"keydown","count":key_count},
            {"type":"keyup","count":key_count},
        ],
        "jsData": {
            "userAgent": user_agent,
            "platform": platform,
            "language": language,
            "languages": ["en-US","en"],
            "hardwareConcurrency": hardware_concurrency,
            "deviceMemory": device_memory,
            "maxTouchPoints": max_touch_points,
            "colorDepth": color_depth,
            "pixelRatio": pixel_ratio,
            "screenWidth": screen_w,
            "screenHeight": screen_h,
            "innerWidth": screen_w,
            "innerHeight": screen_h - 120,
            "canvasHash": canvas_hash,
            "webGLRenderer": webgl_renderer,
            "webGLVendor": "Google Inc. (NVIDIA)",
            "timezone": "America/New_York",
            "timezoneOffset": 300,
            "plugins": "PDF Viewer::Portable Document Format::application/pdf::pdf,Chrome PDF Viewer::Portable Document Format::application/pdf::pdf",
            "cookieEnabled": true,
            "doNotTrack": serde_json::Value::Null,
        },
        "navigationData": {
            "navigationStart": nav_start,
            "domContentLoaded": dom_content_loaded,
            "loadEvent": load_event,
        },
        "mouseData": mouse_data,
        "keyData": key_data,
        "tagpu": rng.gen_range(10.0..50.0) as f64,
        "br_h": screen_h - 120,
        "br_w": screen_w,
        "br_oh": screen_h,
        "br_ow": screen_w,
        "ts": timestamp,
    });

    Ok(sensor.to_string())
}
