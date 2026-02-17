use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use uuid::Uuid;

use super::protocol::{SidecarRequest, SidecarResponse};

/// Manages multiple sidecar sessions for concurrent workers
pub struct SessionPool {
    sidecar_tx: mpsc::Sender<(SidecarRequest, oneshot::Sender<SidecarResponse>)>,
    sessions: Arc<Mutex<HashMap<String, SessionInfo>>>,
}

#[allow(dead_code)]
struct SessionInfo {
    browser: String,
    proxy: Option<String>,
}

impl SessionPool {
    pub fn new(sidecar_tx: mpsc::Sender<(SidecarRequest, oneshot::Sender<SidecarResponse>)>) -> Self {
        Self {
            sidecar_tx,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, browser: &str, proxy: Option<&str>) -> crate::error::Result<String> {
        let session_id = Uuid::new_v4().to_string();

        let req = SidecarRequest {
            id: Uuid::new_v4().to_string(),
            action: "new_session".into(),
            session: session_id.clone(),
            method: None,
            url: None,
            headers: None,
            body: None,
            timeout: None,
            proxy: proxy.map(|s| s.to_string()),
            browser: Some(browser.to_string()),
            ja3: None,
            http2fp: None,
            follow_redirects: Some(true),
            max_redirects: Some(8),
        };

        let (resp_tx, resp_rx) = oneshot::channel();
        self.sidecar_tx.send((req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("Channel closed".into()))?;

        let resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("Response channel closed".into()))?;

        if let Some(err) = resp.error {
            if !err.is_empty() {
                return Err(crate::error::AppError::Sidecar(err));
            }
        }

        self.sessions.lock().await.insert(session_id.clone(), SessionInfo {
            browser: browser.to_string(),
            proxy: proxy.map(|s| s.to_string()),
        });

        Ok(session_id)
    }

    pub async fn close_session(&self, session_id: &str) -> crate::error::Result<()> {
        let req = SidecarRequest {
            id: Uuid::new_v4().to_string(),
            action: "close_session".into(),
            session: session_id.to_string(),
            method: None,
            url: None,
            headers: None,
            body: None,
            timeout: None,
            proxy: None,
            browser: None,
            ja3: None,
            http2fp: None,
            follow_redirects: None,
            max_redirects: None,
        };

        let (resp_tx, _resp_rx) = oneshot::channel();
        let _ = self.sidecar_tx.send((req, resp_tx)).await;
        self.sessions.lock().await.remove(session_id);
        Ok(())
    }

    pub fn get_sender(&self) -> mpsc::Sender<(SidecarRequest, oneshot::Sender<SidecarResponse>)> {
        self.sidecar_tx.clone()
    }
}
