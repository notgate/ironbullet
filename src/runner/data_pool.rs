use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub struct DataPool {
    lines: Vec<String>,
    index: AtomicUsize,
    retry_queue: Mutex<Vec<String>>,
}

impl DataPool {
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            index: AtomicUsize::new(0),
            retry_queue: Mutex::new(Vec::new()),
        }
    }

    pub fn from_file(path: &str, skip_empty: bool) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<String> = content.lines()
            .filter(|l| !skip_empty || !l.trim().is_empty())
            .map(|l| l.to_string())
            .collect();
        Ok(Self::new(lines))
    }

    pub fn next_line(&self) -> Option<String> {
        // Check retry queue first
        if let Ok(mut queue) = self.retry_queue.lock() {
            if let Some(line) = queue.pop() {
                return Some(line);
            }
        }

        let idx = self.index.fetch_add(1, Ordering::Relaxed);
        self.lines.get(idx).cloned()
    }

    pub fn return_line(&self, line: String) {
        if let Ok(mut queue) = self.retry_queue.lock() {
            queue.push(line);
        }
    }

    pub fn total(&self) -> usize {
        self.lines.len()
    }

    pub fn remaining(&self) -> usize {
        let idx = self.index.load(Ordering::Relaxed);
        let retry_count = self.retry_queue.lock().map(|q| q.len()).unwrap_or(0);
        self.lines.len().saturating_sub(idx) + retry_count
    }
}
