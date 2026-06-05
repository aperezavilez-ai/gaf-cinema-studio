use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use uuid::Uuid;

use crate::error::{CinemaError, Result};
use crate::event_bus::{CinemaEvent, EventBus};

#[derive(Debug, Clone)]
pub struct ProxyJob {
    pub id: Uuid,
    pub project_id: Uuid,
    pub media_id: Uuid,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyJobResult {
    Completed { proxy_path: PathBuf },
    Failed { error: String },
}

struct QueueInner {
    pending: VecDeque<ProxyJob>,
    running: bool,
}

pub struct ProxyQueue {
    inner: Arc<(Mutex<QueueInner>, Condvar)>,
    event_bus: EventBus,
    project_dir: PathBuf,
}

impl ProxyQueue {
    pub fn new(project_dir: impl Into<PathBuf>, event_bus: EventBus) -> Self {
        Self {
            inner: Arc::new((Mutex::new(QueueInner {
                pending: VecDeque::new(),
                running: false,
            }), Condvar::new())),
            event_bus,
            project_dir: project_dir.into(),
        }
    }

    pub fn enqueue(&self, job: ProxyJob) -> Result<()> {
        let (lock, cvar) = &*self.inner;
        let mut inner = lock.lock().map_err(|e| {
            CinemaError::Storage(format!("proxy queue lock poisoned: {e}"))
        })?;

        inner.pending.push_back(job.clone());
        cvar.notify_one();

        self.event_bus.emit(CinemaEvent::ProxyGenerationStarted {
            project_id: job.project_id,
            media_id: job.media_id,
        });

        if !inner.running {
            inner.running = true;
            drop(inner);
            self.spawn_worker();
        }

        Ok(())
    }

    pub fn pending_count(&self) -> usize {
        self.inner
            .0
            .lock()
            .map(|i| i.pending.len())
            .unwrap_or(0)
    }

    fn spawn_worker(&self) {
        let inner = Arc::clone(&self.inner);
        let event_bus = self.event_bus.clone();
        let project_dir = self.project_dir.clone();

        thread::spawn(move || {
            loop {
                let job = {
                    let (lock, cvar) = &*inner;
                    let mut state = lock.lock().unwrap();
                    while state.pending.is_empty() {
                        state.running = false;
                        cvar.notify_all();
                        return;
                    }
                    state.pending.pop_front().unwrap()
                };

                let result = process_proxy_job(&project_dir, &job);

                match result {
                    Ok(path) => {
                        event_bus.emit(CinemaEvent::ProxyGenerationCompleted {
                            project_id: job.project_id,
                            media_id: job.media_id,
                            proxy_path: path.display().to_string(),
                        });
                    }
                    Err(e) => {
                        event_bus.emit(CinemaEvent::ProxyGenerationFailed {
                            project_id: job.project_id,
                            media_id: job.media_id,
                            error: e.to_string(),
                        });
                    }
                }

                // Simulate transcode work without blocking caller
                thread::sleep(Duration::from_millis(10));
            }
        });
    }
}

/// Phase 1 stub: creates a placeholder proxy file. Phase 2 replaces with FFmpeg.
fn process_proxy_job(project_dir: &PathBuf, job: &ProxyJob) -> Result<PathBuf> {
    let proxies_dir = project_dir.join("proxies");
    std::fs::create_dir_all(&proxies_dir)?;

    let proxy_path = proxies_dir.join(format!("{}_proxy.mp4", job.media_id));

    // Stub: copy source as proxy placeholder (real transcode in Phase 2)
    if job.source_path.exists() {
        std::fs::copy(&job.source_path, &proxy_path).map_err(|e| {
            CinemaError::Storage(format!("proxy copy failed: {e}"))
        })?;
    } else {
        std::fs::write(&proxy_path, b"PROXY_STUB").map_err(|e| {
            CinemaError::Storage(format!("proxy stub write failed: {e}"))
        })?;
    }

    Ok(proxy_path)
}

impl Clone for ProxyQueue {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            event_bus: self.event_bus.clone(),
            project_dir: self.project_dir.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::EventSubscriber;
    use std::sync::Mutex as StdMutex;
    use tempfile::TempDir;

    struct EventCollector {
        events: Arc<StdMutex<Vec<CinemaEvent>>>,
    }

    impl EventSubscriber for EventCollector {
        fn on_event(&self, event: &CinemaEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    #[test]
    fn proxy_queue_processes_job() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("src.mp4");
        std::fs::write(&source, b"video").unwrap();

        let bus = EventBus::new();
        let events = Arc::new(StdMutex::new(Vec::new()));
        bus.subscribe(Arc::new(EventCollector {
            events: events.clone(),
        }));

        let queue = ProxyQueue::new(tmp.path(), bus);
        let project_id = Uuid::new_v4();
        let media_id = Uuid::new_v4();

        queue
            .enqueue(ProxyJob {
                id: Uuid::new_v4(),
                project_id,
                media_id,
                source_path: source,
            })
            .unwrap();

        // Wait for background worker
        thread::sleep(Duration::from_millis(200));

        let log = events.lock().unwrap();
        assert!(log.iter().any(|e| matches!(e, CinemaEvent::ProxyGenerationStarted { .. })));
    }
}
