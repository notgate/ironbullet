use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

pub struct DataPool {
    lines: Vec<String>,
    index: AtomicUsize,
    retry_queue: Mutex<Vec<(String, u32)>>,
    /// Credentials that exhausted max_retries due to transient errors.
    /// After the main pool drains, these are replayed for one final pass.
    error_queue: Mutex<Vec<String>>,
    /// Whether the error replay pass has already been triggered.
    error_replayed: AtomicBool,
}

impl DataPool {
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            index: AtomicUsize::new(0),
            retry_queue: Mutex::new(Vec::new()),
            error_queue: Mutex::new(Vec::new()),
            error_replayed: AtomicBool::new(false),
        }
    }

    pub fn with_limits(lines: Vec<String>, skip: usize, take: usize) -> Self {
        Self::new(apply_limits(lines, skip, take))
    }

    pub fn from_file(path: &str, skip_empty: bool) -> std::io::Result<Self> {
        Self::from_file_with_limits(path, skip_empty, 0, 0)
    }

    pub fn from_file_with_limits(
        path: &str,
        skip_empty: bool,
        skip: usize,
        take: usize,
    ) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<String> = content
            .lines()
            .filter(|l| !skip_empty || !l.trim().is_empty())
            .map(|l| l.to_string())
            .collect();
        Ok(Self::with_limits(lines, skip, take))
    }

    pub fn next_line(&self) -> Option<(String, u32)> {
        // 1) Prioritise retry queue (credentials that failed transiently)
        if let Ok(mut queue) = self.retry_queue.lock() {
            if let Some(entry) = queue.pop() {
                return Some(entry);
            }
        }
        // 2) Main sequential pool
        let idx = self.index.fetch_add(1, Ordering::Relaxed);
        if let Some(l) = self.lines.get(idx) {
            return Some((l.clone(), 0));
        }
        // 3) Main pool exhausted — replay error queue once (issue #64).
        //    Credentials that hit max_retries due to network/proxy errors get
        //    one final chance with fresh proxies before being permanently dropped.
        if !self.error_replayed.swap(true, Ordering::SeqCst) {
            if let Ok(mut eq) = self.error_queue.lock() {
                if !eq.is_empty() {
                    let mut rq = self.retry_queue.lock().unwrap_or_else(|e| e.into_inner());
                    let count = eq.len();
                    for line in eq.drain(..) {
                        rq.push((line, 0)); // retry_count=0: fresh start
                    }
                    eprintln!(
                        "[data_pool] replaying {} errored credentials for final pass",
                        count
                    );
                    return rq.pop();
                }
            }
        }
        None
    }

    pub fn return_line(&self, line: String, retry_count: u32) {
        if let Ok(mut queue) = self.retry_queue.lock() {
            queue.push((line, retry_count));
        }
    }

    /// Stash a credential that exhausted max_retries due to transient errors.
    /// These will be replayed once the main pool drains.
    pub fn stash_error(&self, line: String) {
        if let Ok(mut queue) = self.error_queue.lock() {
            queue.push(line);
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

fn apply_limits(lines: Vec<String>, skip: usize, take: usize) -> Vec<String> {
    let iter = lines.into_iter().skip(skip);

    if take == 0 {
        iter.collect()
    } else {
        iter.take(take).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::DataPool;

    #[test]
    fn with_limits_skips_prefix_before_processing() {
        let pool = DataPool::with_limits(
            vec![
                "line1".into(),
                "line2".into(),
                "line3".into(),
                "line4".into(),
            ],
            2,
            0,
        );

        assert_eq!(pool.total(), 2);
        assert_eq!(pool.next_line(), Some(("line3".into(), 0)));
        assert_eq!(pool.next_line(), Some(("line4".into(), 0)));
        assert_eq!(pool.next_line(), None);
    }

    #[test]
    fn with_limits_applies_take_after_skip() {
        let pool = DataPool::with_limits(
            vec![
                "line1".into(),
                "line2".into(),
                "line3".into(),
                "line4".into(),
            ],
            1,
            2,
        );

        assert_eq!(pool.total(), 2);
        assert_eq!(pool.next_line(), Some(("line2".into(), 0)));
        assert_eq!(pool.next_line(), Some(("line3".into(), 0)));
        assert_eq!(pool.next_line(), None);
    }
}
