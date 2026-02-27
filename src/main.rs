#![windows_subsystem = "windows"]

mod ipc;

use std::sync::Arc;
use tokio::sync::Mutex;

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::{Icon, WindowBuilder};
use raw_window_handle::HasWindowHandle;
use wry::WebViewBuilder;
use include_dir::{include_dir, Dir};

use ironbullet::config::{load_config, save_config};
use ipc::{AppState, IpcCmd};

/// Check if WebView2 runtime is installed (Windows only).
/// Checks HKCU and HKLM for the WebView2 package registry key.
#[cfg(target_os = "windows")]
fn check_webview2_available() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::System::Registry::{RegOpenKeyExW, RegCloseKey, HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER, KEY_READ};
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;

    let subkeys = [
        r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
        r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
        r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{2CD8A007-E189-409D-A2C8-9AF4EF3C72AA}",
    ];

    let hives = [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER];

    for hive in hives {
        for subkey in &subkeys {
            let wide: Vec<u16> = OsStr::new(subkey).encode_wide().chain(std::iter::once(0)).collect();
            let mut hkey = 0isize;
            let ret = unsafe { RegOpenKeyExW(hive, wide.as_ptr(), 0, KEY_READ, &mut hkey) };
            if ret == ERROR_SUCCESS as i32 {
                unsafe { RegCloseKey(hkey); }
                return true;
            }
        }
    }

    // Also check if the DLL is loadable as a fallback
    false
}

/// Native Win32 title bar drag via WM_NCHITTEST.
/// Returns HTCAPTION for the top 28px (title bar) so Windows handles
/// drag natively — no IPC round-trip, works even with webview dialog overlays.
#[cfg(target_os = "windows")]
mod win32_titlebar {
    use std::sync::atomic::{AtomicIsize, Ordering};
    use windows_sys::Win32::Foundation::{HWND, WPARAM, LPARAM, LRESULT, RECT};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetWindowLongPtrW, CallWindowProcW, GetWindowRect,
        GWLP_WNDPROC, WM_NCHITTEST, HTCLIENT, HTCAPTION, WNDPROC,
    };
    use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;

    static ORIGINAL_WNDPROC: AtomicIsize = AtomicIsize::new(0);

    unsafe extern "system" fn custom_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let orig: WNDPROC = std::mem::transmute(ORIGINAL_WNDPROC.load(Ordering::Relaxed));
        if msg == WM_NCHITTEST {
            let result = CallWindowProcW(orig, hwnd, msg, wparam, lparam);
            if result == HTCLIENT as isize {
                let x = (lparam & 0xFFFF) as i16 as i32;
                let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
                let mut rect: RECT = std::mem::zeroed();
                GetWindowRect(hwnd, &mut rect);
                let dpi = GetDpiForWindow(hwnd);
                let scale = dpi as f64 / 96.0;
                // 28 CSS px title bar, 3×38=114 CSS px chrome buttons on the right
                let title_h = (28.0 * scale) as i32;
                let buttons_w = (114.0 * scale) as i32;
                if y - rect.top < title_h && x < rect.right - buttons_w {
                    return HTCAPTION as isize;
                }
            }
            return result;
        }
        CallWindowProcW(orig, hwnd, msg, wparam, lparam)
    }

    pub unsafe fn install(hwnd: HWND) {
        let orig = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, custom_wndproc as isize);
        ORIGINAL_WNDPROC.store(orig, Ordering::Relaxed);
    }
}

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

/// Check if we should run in CLI mode (any --config or --help arg present)
fn is_cli_mode() -> bool {
    std::env::args().any(|a| a == "--config" || a == "-c" || a == "--help" || a == "-h")
}

/// Attach to parent console on Windows so CLI output is visible.
/// Required because #![windows_subsystem = "windows"] hides the console.
#[cfg(target_os = "windows")]
fn attach_console() {
    unsafe {
        windows_sys::Win32::System::Console::AttachConsole(
            windows_sys::Win32::System::Console::ATTACH_PARENT_PROCESS,
        );
    }
}

#[cfg(not(target_os = "windows"))]
fn attach_console() {}

fn main() {
    if is_cli_mode() {
        attach_console();
        run_cli();
    } else {
        run_gui();
    }
}

fn run_cli() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = match ironbullet::cli::parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {}", e);
            eprintln!("run with --help for usage");
            std::process::exit(1);
        }
    };

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        if let Err(e) = ironbullet::cli::run(cli).await {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    });
}

/// Position window on screen. Computes physical window size from known logical
/// dimensions × DPI (avoids unreliable `outer_size()` on freshly-created windows).
///
/// 1. If saved position exists AND the window's center would be on a visible
///    monitor → restore it with PhysicalPosition.
/// 2. Otherwise → center on the primary monitor's work area (rcWork, excludes
///    taskbar) — the same approach as WPF's WindowStartupLocation.CenterScreen.
fn position_window(
    window: &tao::window::Window,
    saved_x: Option<i32>,
    saved_y: Option<i32>,
    logical_w: f64,
    logical_h: f64,
) {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::POINT;
        use windows_sys::Win32::Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromPoint, MONITORINFO,
            MONITOR_DEFAULTTONULL, MONITOR_DEFAULTTOPRIMARY,
        };

        // Physical window size from known logical dimensions
        let scale = window.scale_factor();
        let win_w = (logical_w * scale) as i32;
        let win_h = (logical_h * scale) as i32;

        // Try restoring saved position — validate that the window's CENTER
        // would land on a visible monitor (not just the top-left corner)
        if let (Some(x), Some(y)) = (saved_x, saved_y) {
            let cx = x + win_w / 2;
            let cy = y + win_h / 2;
            let center_visible = unsafe {
                !MonitorFromPoint(POINT { x: cx, y: cy }, MONITOR_DEFAULTTONULL).is_null()
            };
            if center_visible {
                window.set_outer_position(tao::dpi::PhysicalPosition::new(x, y));
                return;
            }
        }

        // Center on primary monitor's work area.
        // (0, 0) is always on the primary monitor in Windows coordinate space.
        unsafe {
            let hmon = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
            if !hmon.is_null() {
                let mut info: MONITORINFO = std::mem::zeroed();
                info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                if GetMonitorInfoW(hmon, &mut info) != 0 {
                    let rc = info.rcWork;
                    let work_w = rc.right - rc.left;
                    let work_h = rc.bottom - rc.top;
                    let x = rc.left + (work_w - win_w) / 2;
                    let y = rc.top + (work_h - win_h) / 2;
                    window.set_outer_position(tao::dpi::PhysicalPosition::new(x, y));
                }
            }
        }
        return;
    }

    // Non-Windows fallback
    #[cfg(not(target_os = "windows"))]
    {
        if let (Some(x), Some(y)) = (saved_x, saved_y) {
            window.set_outer_position(tao::dpi::PhysicalPosition::new(x, y));
        } else if let Some(monitor) = window.primary_monitor().or_else(|| window.current_monitor()) {
            let mon_size = monitor.size();
            let mon_pos = monitor.position();
            let scale = window.scale_factor();
            let win_w = (logical_w * scale) as i32;
            let win_h = (logical_h * scale) as i32;
            let x = mon_pos.x + ((mon_size.width as i32 - win_w) / 2);
            let y = mon_pos.y + ((mon_size.height as i32 - win_h) / 2);
            window.set_outer_position(tao::dpi::PhysicalPosition::new(x, y));
        }
    }
}

fn run_gui() {
    // Clean up old binary from previous update
    if let Ok(exe) = std::env::current_exe() {
        let old = exe.with_extension("old.exe");
        if old.exists() {
            let _ = std::fs::remove_file(&old);
        }
    }

    let cfg = load_config();

    // Start tokio runtime in background
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let _guard = rt.enter();

    let event_loop: EventLoop<Evt> = tao::event_loop::EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let mut wb = WindowBuilder::new()
        .with_title("Ironbullet")
        .with_inner_size(tao::dpi::LogicalSize::new(cfg.window_width, cfg.window_height))
        .with_decorations(false);

    // Load app icon from embedded PNG
    if let Ok(img) = image::load_from_memory(include_bytes!("../data/IMGS/notextlogo.png")) {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        if let Ok(icon) = Icon::from_rgba(rgba.into_raw(), w, h) {
            wb = wb.with_window_icon(Some(icon));
        }
    }

    let window = wb.build(&event_loop).expect("Failed to create window");

    // Restore saved position or center on work area (WPF CenterScreen style)
    position_window(&window, cfg.window_x, cfg.window_y, cfg.window_width, cfg.window_height);

    // Windows-specific: disable rounded corners + install native title bar drag
    #[cfg(target_os = "windows")]
    {
        if let Ok(handle) = window.window_handle() {
            if let raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() {
                let hwnd = win32.hwnd.get() as *mut std::ffi::c_void;
                // Disable Windows 11 rounded corners
                use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND};
                let preference: u32 = DWMWCP_DONOTROUND as u32;
                unsafe {
                    DwmSetWindowAttribute(
                        hwnd,
                        DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                        &preference as *const _ as *const _,
                        std::mem::size_of_val(&preference) as u32,
                    );
                    // Native title bar drag — works even with webview dialog overlays
                    win32_titlebar::install(hwnd);
                }
            }
        }
    }

    let state = Arc::new(Mutex::new(AppState::new()));
    let ipc_proxy = proxy.clone();
    let ipc_state = state.clone();

    // ── WebView2 availability check (Windows only) ──────────────────────────────
    #[cfg(target_os = "windows")]
    {
        let wv2_available = check_webview2_available();
        if !wv2_available {
            use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONERROR};
            let title: Vec<u16> = "IronBullet — Missing Dependency\0".encode_utf16().collect();
            let msg: Vec<u16> = concat!(
                "WebView2 runtime was not found.\n\n",
                "IronBullet requires the Microsoft Edge WebView2 runtime to run.\n",
                "Please install it from:\n\n",
                "https://go.microsoft.com/fwlink/p/?LinkId=2124703\n\n",
                "After installing, restart IronBullet.\0"
            ).encode_utf16().collect();
            unsafe {
                MessageBoxW(0, msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
            }
            std::process::exit(1);
        }
    }

    let webview = WebViewBuilder::new()
        .with_devtools(true)
        .with_custom_protocol("ironbullet".into(), move |_wv, req| {
            use std::borrow::Cow;
            let uri = req.uri().to_string();
            let path = uri.strip_prefix("ironbullet://localhost").unwrap_or(&uri);
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
        .with_url("ironbullet://localhost/")
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

                // Spawn IPC handlers async to prevent blocking the webview
                let eval_proxy = proxy.clone();
                let state_clone = ipc_state.clone();
                tokio::spawn(async move {
                    ipc::handle_ipc_cmd(&cmd, &state_clone, move |js| {
                        let _ = eval_proxy.send_event(Evt::EvalJs(js));
                    });
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
                // Redirect to frontend so it can prompt for unsaved tabs
                let _ = webview.evaluate_script(
                    "window.dispatchEvent(new Event('native-close-requested'))"
                );
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
