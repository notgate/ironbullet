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

pub struct SidecarManager {
    process: Option<Child>,
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<SidecarResponse>>>>,
    writer_tx: Option<mpsc::Sender<String>>,
}

impl SidecarManager {
    pub fn new() -> Self {
        Self {
            process: None,
            pending: Arc::new(Mutex::new(HashMap::new())),
            writer_tx: None,
        }
    }

    pub async fn start(&mut self, sidecar_path: &str) -> crate::error::Result<mpsc::Sender<(SidecarRequest, oneshot::Sender<SidecarResponse>)>> {
        let mut child = Command::new(sidecar_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| crate::error::AppError::Sidecar(format!("Failed to spawn sidecar: {}", e)))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| crate::error::AppError::Sidecar("No stdin".into()))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| crate::error::AppError::Sidecar("No stdout".into()))?;

        // Writer channel
        let (writer_tx, mut writer_rx) = mpsc::channel::<String>(1024);
        let mut stdin = stdin;
        tokio::spawn(async move {
            while let Some(line) = writer_rx.recv().await {
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.write_all(b"\n").await.is_err() {
                    break;
                }
                let _ = stdin.flush().await;
            }
        });

        // Reader task
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

        // Request channel
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
        Ok(req_tx)
    }

    pub async fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill().await;
        }
        self.writer_tx = None;
    }

    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }
}
