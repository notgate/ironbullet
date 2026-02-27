pub mod native;
pub mod protocol;
pub mod session;

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, Mutex};

use protocol::{SidecarRequest, SidecarResponse};

type ReqTx = mpsc::Sender<(SidecarRequest, oneshot::Sender<SidecarResponse>)>;

pub struct SidecarManager {
    process: Option<Child>,
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<SidecarResponse>>>>,
    writer_tx: Option<mpsc::Sender<String>>,
    /// Stored so jobs can get a sender without restarting the sidecar
    req_tx: Option<ReqTx>,
}

impl SidecarManager {
    pub fn new() -> Self {
        Self {
            process: None,
            pending: Arc::new(Mutex::new(HashMap::new())),
            writer_tx: None,
            req_tx: None,
        }
    }

    /// Start the sidecar process. Returns a cloneable sender for sending requests.
    pub async fn start(&mut self, sidecar_path: &str) -> crate::error::Result<ReqTx> {
        // Pre-flight: verify the sidecar file exists before spawning.
        // This gives a clear error instead of the generic OS "not found" message.
        let path = std::path::Path::new(sidecar_path);
        if !path.exists() {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|e| e.parent().map(|p| p.display().to_string()))
                .unwrap_or_else(|| String::from("<unknown>"));
            let sidecar_name = if cfg!(target_os = "windows") {
                "reqflow-sidecar.exe"
            } else {
                "reqflow-sidecar"
            };
            return Err(crate::error::AppError::Sidecar(format!(
                "Sidecar not found at '{}'.\n\
                 Make sure {} is in the same folder as the IronBullet executable.\n\
                 Executable directory: {}",
                sidecar_path, sidecar_name, exe_dir
            )));
        }

        // Set the working directory to the sidecar's own folder so that any
        // relative paths the sidecar uses resolve correctly.
        let sidecar_dir = path.parent().unwrap_or(std::path::Path::new("."));

        let mut child = Command::new(sidecar_path)
            .current_dir(sidecar_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| crate::error::AppError::Sidecar(format!(
                "Failed to spawn sidecar '{}': {}\n\
                 (file exists={}, is_executable={})",
                sidecar_path, e,
                path.exists(),
                path.metadata().map(|m| {
                    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; m.permissions().mode() & 0o111 != 0 }
                    #[cfg(not(unix))] { true }
                }).unwrap_or(false)
            )))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| crate::error::AppError::Sidecar("No stdin".into()))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| crate::error::AppError::Sidecar("No stdout".into()))?;

        // Writer channel
        let (writer_tx, mut writer_rx) = mpsc::channel::<String>(1024);
        let mut stdin = stdin;
        tokio::spawn(async move {
            while let Some(line) = writer_rx.recv().await {
                if stdin.write_all(line.as_bytes()).await.is_err() { break; }
                if stdin.write_all(b"\n").await.is_err() { break; }
                let _ = stdin.flush().await;
            }
        });

        // Reader task — routes responses back to waiting callers
        let pending = self.pending.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(resp) = serde_json::from_str::<SidecarResponse>(&line) {
                    let mut map = pending.lock().await;
                    if let Some(tx) = map.remove(&resp.id) {
                        let _ = tx.send(resp);
                    }
                }
            }
        });

        // Request multiplexer channel — serialize requests onto the writer
        let (req_tx, mut req_rx) = mpsc::channel::<(SidecarRequest, oneshot::Sender<SidecarResponse>)>(1024);
        let writer_tx2 = writer_tx.clone();
        let pending2 = self.pending.clone();
        tokio::spawn(async move {
            while let Some((req, resp_tx)) = req_rx.recv().await {
                let id = req.id.clone();
                pending2.lock().await.insert(id, resp_tx);
                if let Ok(json) = serde_json::to_string(&req) {
                    let _ = writer_tx2.send(json).await;
                }
            }
        });

        self.process = Some(child);
        self.writer_tx = Some(writer_tx);
        self.req_tx = Some(req_tx.clone());
        Ok(req_tx)
    }

    /// Return a sender for an already-running sidecar, or None if not running.
    pub fn get_req_tx(&self) -> Option<ReqTx> {
        self.req_tx.clone()
    }

    /// Get or start: reuse existing sender if the sidecar is running, otherwise start fresh.
    pub async fn get_or_start(&mut self, sidecar_path: &str) -> crate::error::Result<ReqTx> {
        if self.is_running() {
            if let Some(tx) = self.get_req_tx() {
                return Ok(tx);
            }
        }
        // Not running or no tx — start fresh
        self.stop().await;
        self.start(sidecar_path).await
    }

    pub async fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill().await;
        }
        self.writer_tx = None;
        self.req_tx = None;
    }

    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }
}
