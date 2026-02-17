#![windows_subsystem = "windows"]

mod ipc;

use std::sync::Arc;
use tokio::sync::Mutex;

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use raw_window_handle::HasWindowHandle;
use wry::WebViewBuilder;
use include_dir::{include_dir, Dir};

use reqflow::config::{load_config, save_config};
use ipc::{AppState, IpcCmd};

static GUI_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/gui/build");

fn mime_for(path: &str) -> &'static str {
    if path.ends_with(".html") { "text/html" }
    else if path.ends_with(".js") { "application/javascript" }
    else if path.ends_with(".css") { "text/css" }
    else if path.ends_with(".svg") { "image/svg+xml" }
    else if path.ends_with(".png") { "image/png" }
    else if path.ends_with(".ico") { "image/x-icon" }
    else if path.ends_with(".json") { "application/json" }
    else if path.ends_with(".woff2") { "font/woff2" }
    else if path.ends_with(".woff") { "font/woff" }
    else if path.ends_with(".ttf") { "font/ttf" }
    else { "application/octet-stream" }
}

enum Evt {
    EvalJs(String),
    DragWindow,
    MinimizeWindow,
    MaximizeWindow,
    CloseWindow,
}

fn main() {
    let cfg = load_config();

    // Start tokio runtime in background
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let _guard = rt.enter();

    let event_loop: EventLoop<Evt> = tao::event_loop::EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let mut wb = WindowBuilder::new()
        .with_title("reqflow")
        .with_inner_size(tao::dpi::LogicalSize::new(cfg.window_width, cfg.window_height))
        .with_decorations(false);

    if let (Some(x), Some(y)) = (cfg.window_x, cfg.window_y) {
        wb = wb.with_position(tao::dpi::LogicalPosition::new(x, y));
    }

    let window = wb.build(&event_loop).expect("Failed to create window");

    // Center on screen if no saved position
    if cfg.window_x.is_none() || cfg.window_y.is_none() {
        if let Some(monitor) = window.current_monitor() {
            let mon_size = monitor.size();
            let mon_pos = monitor.position();
            let win_size = window.outer_size();
            let x = mon_pos.x + ((mon_size.width as i32 - win_size.width as i32) / 2);
            let y = mon_pos.y + ((mon_size.height as i32 - win_size.height as i32) / 2);
            window.set_outer_position(tao::dpi::PhysicalPosition::new(x.max(0), y.max(0)));
        }
    }

    // Disable Windows 11 rounded corners
    #[cfg(target_os = "windows")]
    {
        if let Ok(handle) = window.window_handle() {
            if let raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() {
                use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND};
                let preference: u32 = DWMWCP_DONOTROUND as u32;
                unsafe {
                    DwmSetWindowAttribute(
                        win32.hwnd.get() as _,
                        DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                        &preference as *const _ as *const _,
                        std::mem::size_of_val(&preference) as u32,
                    );
                }
            }
        }
    }

    // Center on screen on first launch (no saved position)
    if cfg.window_x.is_none() && cfg.window_y.is_none() {
        if let Some(monitor) = window.current_monitor() {
            let monitor_size = monitor.size();
            let window_size = window.outer_size();
            let x = (monitor_size.width.saturating_sub(window_size.width)) / 2;
            let y = (monitor_size.height.saturating_sub(window_size.height)) / 2;
            window.set_outer_position(tao::dpi::PhysicalPosition::new(x, y));
        }
    }

    let state = Arc::new(Mutex::new(AppState::new()));
    let ipc_proxy = proxy.clone();
    let ipc_state = state.clone();

    let webview = WebViewBuilder::new()
        .with_devtools(true)
        .with_custom_protocol("reqflow".into(), move |_wv, req| {
            use std::borrow::Cow;
            let uri = req.uri().to_string();
            let path = uri.strip_prefix("reqflow://localhost").unwrap_or(&uri);
            let path = if path.is_empty() || path == "/" { "/index.html" } else { path };
            let path = path.trim_start_matches('/');
            // Strip query string
            let path = path.split('?').next().unwrap_or(path);
            let (body, mime): (Cow<'static, [u8]>, &str) = match GUI_DIR.get_file(path) {
                Some(f) => (Cow::Borrowed(f.contents()), mime_for(path)),
                None => {
                    match GUI_DIR.get_file("index.html") {
                        Some(f) => (Cow::Borrowed(f.contents()), "text/html"),
                        None => (Cow::Borrowed(b"404 Not Found"), "text/plain"),
                    }
                }
            };
            wry::http::Response::builder()
                .header("Content-Type", mime)
                .header("Access-Control-Allow-Origin", "*")
                .header("Cache-Control", "no-cache, no-store, must-revalidate")
                .body(body)
                .unwrap()
        })
        .with_url("reqflow://localhost/")
        .with_ipc_handler(move |req: wry::http::Request<String>| {
            let body = req.body();
            let proxy = ipc_proxy.clone();

            // Handle window chrome commands directly
            if let Ok(cmd) = serde_json::from_str::<IpcCmd>(body) {
                match cmd.cmd.as_str() {
                    "drag" => {
                        let _ = proxy.send_event(Evt::DragWindow);
                        return;
                    }
                    "minimize" => {
                        let _ = proxy.send_event(Evt::MinimizeWindow);
                        return;
                    }
                    "maximize" => {
                        let _ = proxy.send_event(Evt::MaximizeWindow);
                        return;
                    }
                    "close" => {
                        let _ = proxy.send_event(Evt::CloseWindow);
                        return;
                    }
                    _ => {}
                }

                let eval_proxy = proxy.clone();
                ipc::handle_ipc_cmd(&cmd, &ipc_state, move |js| {
                    let _ = eval_proxy.send_event(Evt::EvalJs(js));
                });
            }
        })
        .build(&window)
        .expect("Failed to create webview");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                // Save window position & size
                let size = window.inner_size();
                let pos = window.outer_position().ok();
                let scale = window.scale_factor();
                let rt = tokio::runtime::Handle::try_current();
                if let Ok(handle) = rt {
                    let state = state.clone();
                    handle.spawn(async move {
                        let mut s = state.lock().await;
                        s.config.window_width = size.width as f64 / scale;
                        s.config.window_height = size.height as f64 / scale;
                        if let Some(pos) = pos {
                            s.config.window_x = Some(pos.x);
                            s.config.window_y = Some(pos.y);
                        }
                        save_config(&s.config);
                    });
                }
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(evt) => match evt {
                Evt::EvalJs(js) => {
                    let _ = webview.evaluate_script(&js);
                }
                Evt::DragWindow => {
                    let _ = window.drag_window();
                }
                Evt::MinimizeWindow => {
                    window.set_minimized(true);
                }
                Evt::MaximizeWindow => {
                    window.set_maximized(!window.is_maximized());
                }
                Evt::CloseWindow => {
                    // Save window position & size before closing (same as CloseRequested)
                    let size = window.inner_size();
                    let pos = window.outer_position().ok();
                    let scale = window.scale_factor();
                    let rt = tokio::runtime::Handle::try_current();
                    if let Ok(handle) = rt {
                        let state = state.clone();
                        handle.spawn(async move {
                            let mut s = state.lock().await;
                            s.config.window_width = size.width as f64 / scale;
                            s.config.window_height = size.height as f64 / scale;
                            if let Some(pos) = pos {
                                s.config.window_x = Some(pos.x);
                                s.config.window_y = Some(pos.y);
                            }
                            save_config(&s.config);
                        });
                    }
                    *control_flow = ControlFlow::Exit;
                }
            },
            _ => {}
        }
    });
}
