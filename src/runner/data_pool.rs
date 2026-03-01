use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub struct DataPool {
    lines: Vec<String>,
    index: AtomicUsize,
    retry_queue: Mutex<Vec<(String, u32)>>,
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

    pub fn next_line(&self) -> Option<(String, u32)> {
        if let Ok(mut queue) = self.retry_queue.lock() {
            if let Some(entry) = queue.pop() {
                return Some(entry);
            }
        }
        let idx = self.index.fetch_add(1, Ordering::Relaxed);
        self.lines.get(idx).map(|l| (l.clone(), 0))
    }

    pub fn return_line(&self, line: String, retry_count: u32) {
        if let Ok(mut queue) = self.retry_queue.lock() {
            queue.push((line, retry_count));
        }
    }

    pub fn total(&self) -> usize {
        self.lines.len()
    }

    pub fn consumed(&self) -> usize {
        self.index.load(Ordering::Relaxed).min(self.lines.len())
    }

    pub fn remaining(&self) -> usize {
        let idx = self.index.load(Ordering::Relaxed);
        let retry_count = self.retry_queue.lock().map(|q| q.len()).unwrap_or(0);
        self.lines.len().saturating_sub(idx) + retry_count
    }
}
